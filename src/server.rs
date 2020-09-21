use tonic::{transport::Server, Request, Response, Status};

use mpf::media_controller_server::{MediaController, MediaControllerServer};
use mpf::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::path::PathBuf;

extern crate image;


pub mod mpf {
    tonic::include_proto!("mpf");

    pub fn convert_color(color: Color) -> graphics::types::Color {
        [color.red, color.green, color.blue, color.alpha]
    }

}

#[derive(Debug)]
pub struct MyMediaController {
    scene: Arc<crate::scene::Scene>,
    asset_path: PathBuf
}

#[tonic::async_trait]
impl MediaController for MyMediaController {

    async fn add_slide(&self, _request: Request<SlideAddRequest>) ->
    Result<Response<SlideAddResponse>, Status> {
        let mut next_slide_id = self.scene.next_slide_id.lock().unwrap();
        *next_slide_id += 1;
        let empty_slide = Arc::new(Mutex::new(crate::scene::Slide {
            widgets: vec![]
        }));
        self.scene.slides.lock().unwrap().insert(*next_slide_id, empty_slide);

        Ok(Response::new(SlideAddResponse{
            slide_id: *next_slide_id
        }))     
    }

    async fn show_slide(&self, request: tonic::Request<ShowSlideRequest>) -> 
    Result<tonic::Response<ShowSlideResponse>, tonic::Status> {
        let req = request.into_inner();
        println!("Show slide {}", req.slide_id);
       
        let mut current_slide = self.scene.current_slide.lock().unwrap();
        match self.scene.slides.lock().unwrap().get_mut(&req.slide_id) {
            Some(slide) => {
                *current_slide = slide.clone();

                Ok(Response::new(ShowSlideResponse{}))
            }
            None => {
                Err(Status::invalid_argument("Could not find slide"))
            }
        }
    }

    async fn add_widgets_to_slide(&self, request: tonic::Request<WidgetAddRequest>) ->
    Result<tonic::Response<WidgetAddResponse>, tonic::Status> {
        let req = request.into_inner();
        println!("Slide {} Widget: {:?}", req.slide_id, req.widget);

        let widget = req.widget.ok_or(Status::invalid_argument("Missing Widget Type"))?;
        let color = req.color.ok_or(Status::invalid_argument("Missing Color"))?;
       
        let new_widget = crate::scene::Widget {
            x: req.x,
            y: req.y,
            z: req.z,
            id: req.slide_id,
            color: mpf::convert_color(color),
            widget: match widget {
                mpf::widget_add_request::Widget::TextWidget(widget)  => {
                    crate::scene::WidgetType::Text {
                        text: widget.text
                    }
                },
                mpf::widget_add_request::Widget::RectangleWidget(widget) => {
                    crate::scene::WidgetType::Rectacle {
                        height: widget.height,
                        width: widget.width
                    }
                },
                mpf::widget_add_request::Widget::ImageWidget(widget) => {
                    let path = PathBuf::from(widget.path);

                    let img = image::open(path).map_err(|e| Status::invalid_argument(e.to_string()))?;

                    let img = match img {
                        image::DynamicImage::ImageRgba8(img) => img,
                        img => img.to_rgba()
                    };

                    crate::scene::WidgetType::Image {
                        image: img
                    }
                }
            }
        };

        match self.scene.slides.lock().unwrap().get_mut(&req.slide_id) {
            Some(slide) => {
                slide.lock().unwrap().widgets.push(new_widget);
                Ok(Response::new(WidgetAddResponse{}))
            }
            None => {
                Err(Status::invalid_argument("Could not find slide"))
            }
        }
    }
}

pub async fn serve(scene: Arc<crate::scene::Scene>, asset_path: PathBuf) {
    let addr = "[::1]:50051".parse().unwrap();
    let mc = MyMediaController{
        scene,
        asset_path
    };

    Server::builder()
        .add_service(MediaControllerServer::new(mc))
        .serve(addr)
        .await.unwrap();
}