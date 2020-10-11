use tonic::{transport::Server, Request, Response, Status};

use mpf::media_controller_server::{MediaController, MediaControllerServer};
use mpf::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::path::PathBuf;

use ggez::graphics;

extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;

use self::gst_video::prelude::*;
//use gst::{glib, gst_element_error};
//use gst::{BufferMap, buffer::Readable};

use arc_swap::ArcSwapOption;

extern crate image;


pub mod mpf {
    use ggez::graphics;

    tonic::include_proto!("mpf");

    pub fn convert_color(color: Color) -> graphics::Color {
        graphics::Color::new(color.red, color.green, color.blue, color.alpha)
    }

}

pub struct MyMediaController {
    scene: Arc<crate::scene::Scene>,
    gst_gl_context: gst_gl::GLContext,
}

fn create_video_pipeline(uri: &str, video_memory: Arc<ArcSwapOption<gst::Sample>>) -> gst::Pipeline {
    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("filesrc", None).unwrap();
    let decodebin =
        gst::ElementFactory::make("decodebin", None).unwrap();

    // Tell the filesrc what file to load
    src.set_property("location", &uri).unwrap();

    pipeline.add_many(&[&src, &decodebin]).unwrap();
    gst::Element::link_many(&[&src, &decodebin]).unwrap();

    // Need to move a new reference into the closure.
    // !!ATTENTION!!:
    // It might seem appealing to use pipeline.clone() here, because that greatly
    // simplifies the code within the callback. What this actually does, however, is creating
    // a memory leak. The clone of a pipeline is a new strong reference on the pipeline.
    // Storing this strong reference of the pipeline within the callback (we are moving it in!),
    // which is in turn stored in another strong reference on the pipeline is creating a
    // reference cycle.
    // DO NOT USE pipeline.clone() TO USE THE PIPELINE WITHIN A CALLBACK
    let pipeline_weak = pipeline.downgrade();
    // Connect to decodebin's pad-added signal, that is emitted whenever
    // it found another stream from the input file and found a way to decode it to its raw format.
    // decodebin automatically adds a src-pad for this raw stream, which
    // we can use to build the follow-up pipeline.
    decodebin.connect_pad_added(move |dbin, src_pad| {
        // Here we temporarily retrieve a strong reference on the pipeline from the weak one
        // we moved into this callback.
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => return,
        };

        // Try to detect whether the raw stream decodebin provided us with
        // just now is either audio or video (or none of both, e.g. subtitles).
        let (is_audio, is_video) = {
            let media_type = src_pad.get_current_caps().and_then(|caps| {
                println!("Caps {:?}", caps);
                caps.get_structure(0).map(|s| {
                    let name = s.get_name();
                    (name.starts_with("audio/"), name.starts_with("video/"))
                })
            });
            // TODO: get video resolution here and calculate scale ratio

            match media_type {
                None => {
                    gst::gst_element_warning!(
                        dbin,
                        gst::CoreError::Negotiation,
                        ("Failed to get media type from pad {}", src_pad.get_name())
                    );

                    return;
                }
                Some(media_type) => media_type,
            }
        };

        if is_audio {
            // decodebin found a raw audiostream, so we build the follow-up pipeline to
            // play it on the default audio playback device (using autoaudiosink).
            let queue = gst::ElementFactory::make("queue", None)
            .unwrap();
            let convert = gst::ElementFactory::make("audioconvert", None)
            .unwrap();
            let resample = gst::ElementFactory::make("audioresample", None)
            .unwrap();
            let sink = gst::ElementFactory::make("appsink", Some("audio_sink"))
            .unwrap();

            let elements = &[&queue, &convert, &resample, &sink];
            pipeline.add_many(elements).unwrap();
            gst::Element::link_many(elements).unwrap();

            // !!ATTENTION!!:
            // This is quite important and people forget it often. Without making sure that
            // the new elements have the same state as the pipeline, things will fail later.
            // They would still be in Null state and can't process data.
            for e in elements {
                e.sync_state_with_parent().unwrap()
            }

            // Get the queue element's sink pad and link the decodebin's newly created
            // src pad for the audio stream to it.
            let sink_pad = queue.get_static_pad("sink").expect("queue has no sinkpad");
            src_pad.link(&sink_pad).unwrap();
        }
        if is_video {
            // decodebin found a raw videostream, so we build the follow-up pipeline to
            // display it using the autovideosink.
            let queue = gst::ElementFactory::make("queue", None)
            .unwrap();
            let convert = gst::ElementFactory::make("videoconvert", None)
            .unwrap();
            let scale = gst::ElementFactory::make("videoscale", None)
            .unwrap();
            let sink = gst::ElementFactory::make("appsink", Some("video_sink"))
            .unwrap();

            //convert.set_property("n-threads", &2u32.to_value()).unwrap();
            //queue.set_property("max-size-bytes", &100485760u32.to_value()).unwrap();
            //queue.set_property_from_str("leaky", "upstream");

            // TODO: configure videoscale algorithm (https://gstreamer.freedesktop.org/documentation/videoscale/index.html?gi-language=c)
            // TODO: only instantiate videoscale if configured
            // TODO: set "add-borders" to false for scale

            let elements = &[&queue, &convert, &scale, &sink];
            pipeline.add_many(elements).unwrap();
            gst::Element::link_many(elements).unwrap();

            for e in elements {
                e.sync_state_with_parent().unwrap()
            }

            let sink = sink.downcast::<gst_app::AppSink>().unwrap();
            sink.set_caps(Some(
                &gst::Caps::builder("video/x-raw")
                    .field("format", &gst_video::VideoFormat::Rgba.to_str())
                    .build(),
            ));
            // TODO: set calculated target resolution here

            // Get the queue element's sink pad and link the decodebin's newly created
            // src pad for the video stream to it.
            let sink_pad = queue.get_static_pad("sink").expect("queue has no sinkpad");
            src_pad.link(&sink_pad).unwrap();
            let video_memory = video_memory.clone();
            sink.set_callbacks(gst_app::AppSinkCallbacks::builder().new_sample(move |appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                video_memory.swap(Some(Arc::new(sample)));
                Ok(gst::FlowSuccess::Ok)
            }).build());
        }

    });

    pipeline.set_state(gst::State::Playing).unwrap();
    pipeline
        .get_state(gst::CLOCK_TIME_NONE)
        .0
        .unwrap();
    pipeline
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
       
        let new_widget = crate::scene::Widget {
            x: req.x as f32,
            y: req.y as f32,
            z: req.z,
            id: req.slide_id,
            render_state: crate::scene::RenderState::NoContent,
            update_state: crate::scene::UpdateState::NeedsUpdate,
            widget: match widget {
                mpf::widget_add_request::Widget::LabelWidget(widget)  => {
                    let color = widget.color.ok_or(Status::invalid_argument("Missing Color"))?;
                    crate::scene::WidgetType::Label {
                        text: widget.text,
                        color: mpf::convert_color(color),
                        font_size: widget.font_size as u32,
                        font: None
                    }
                },
                mpf::widget_add_request::Widget::RectangleWidget(widget) => {
                    let color = widget.color.ok_or(Status::invalid_argument("Missing Color"))?;
                    crate::scene::WidgetType::Rectacle {
                        height: widget.height,
                        width: widget.width,
                        color: mpf::convert_color(color),
                    }
                },
                mpf::widget_add_request::Widget::ImageWidget(widget) => {
                    let path = PathBuf::from(widget.path);

                    let img = image::open(path).map_err(|e| Status::invalid_argument(e.to_string()))?;

                    // TODO: calculate target size
                    // TODO: reize here using image::imageops::resize

                    let img = match img {
                        image::DynamicImage::ImageRgba8(img) => img,
                        img => img.to_rgba()
                    };

                    crate::scene::WidgetType::Image {
                        image: img
                    }
                },
                mpf::widget_add_request::Widget::ImageSpriteWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },
                mpf::widget_add_request::Widget::AnimatedImageWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },
                mpf::widget_add_request::Widget::VideoWidget(widget) => {
                    let video_memory = Arc::new(ArcSwapOption::from(None));
                    let pipeline = create_video_pipeline(&widget.path, video_memory.clone());

                    let video_sink = pipeline
                    .get_by_name("video_sink")
                    .expect("Sink element not found")
                    .downcast::<gst_app::AppSink>()
                    .expect("Sink element is expected to be an appsink!");
                    crate::scene::WidgetType::Video {
                        pipeline,
                        video_sink,
                        video_memory
                    }
                },
                mpf::widget_add_request::Widget::DisplayWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },             
                mpf::widget_add_request::Widget::LineWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },             
                mpf::widget_add_request::Widget::PolygonWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },
                mpf::widget_add_request::Widget::BezierWidget(_widget) => {
                    crate::scene::WidgetType::Label {
                        text: "UNSUPPORTED".into(),
                        color: graphics::WHITE,
                        font_size: 32,
                        font: None
                    }
                },
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

pub async fn serve(scene: Arc<crate::scene::Scene>, gst_gl_context: gst_gl::GLContext) {
    let addr = "[::1]:50051".parse().unwrap();
    let mc = MyMediaController{
        scene,
        gst_gl_context
    };

    Server::builder()
        .add_service(MediaControllerServer::new(mc))
        .serve(addr)
        .await.unwrap();
}