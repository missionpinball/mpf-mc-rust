use std::collections::HashMap;
use graphics::context::Context;
use graphics::text::Text;
use crate::graphics::Transformed;
use piston_window::*;
use std::sync::Mutex;
use std::sync::Arc;
use texture::*;

extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;
use self::gst_video::prelude::*;

extern crate image;


#[derive(Debug)]
pub struct Scene {
    pub slides: Mutex<HashMap<u32, Arc<Mutex<Slide>>>>,
    pub current_slide: Mutex<Arc<Mutex<Slide>>>,
    pub next_slide_id: Mutex<u32>
}

impl Scene {
    pub fn new() -> Scene {
        let empty_slide = Arc::new(Mutex::new(Slide {
            widgets: vec![]
        }));

        let slides = Mutex::new(HashMap::new());
        slides.lock().unwrap().insert(0, empty_slide.clone());

        Scene {
            slides,
            current_slide: Mutex::new(empty_slide),
            next_slide_id: Mutex::new(0)
        }
    }
}

#[derive(Debug)]
pub struct Slide {
    pub widgets: Vec<Widget>

}

impl Slide {
    pub fn render(&self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        for widget in &self.widgets {
            widget.draw(c, gl, mc_context);
        }
    }
}

#[derive(Debug)]
pub struct Widget {
    pub x: f64,
    pub y: f64,
    pub z: u32,
    pub id: u32,
    pub widget: WidgetType
}
#[derive(Debug)]
pub enum WidgetType {
    Text {
        text: String,
        color: graphics::types::Color,
    },
    Rectacle {
        width: f64,
        height: f64,
        color: graphics::types::Color,
    },
    Image {
        image: image::RgbaImage
    },
    Video {
        pipeline: gst::Pipeline
    }
}

impl Widget {
    fn draw(&self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        // TODO: add graphics_buffer and only render when something changed

        self.render(c, gl, mc_context)
    }

    fn render(&self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        let transform = c.transform.trans(self.x, self.y);

        match &self.widget {
            WidgetType::Text { text, color} => {
                Text::new_color(*color, 32).draw(
                                text,
                                &mut mc_context.glyphs,
                                &c.draw_state,
                                transform, gl
                            ).unwrap();
            }
            WidgetType::Rectacle { width, height, color } => {
                rectangle(*color, [0.0, 0.0, *width, *height], transform, gl);
            },
            WidgetType::Video { pipeline } => {

                let video_sink = pipeline
                .get_by_name("video_sink")
                .expect("Sink element not found")
                .downcast::<gst_app::AppSink>()
                .expect("Sink element is expected to be an appsink!");
        
                let sample = video_sink.pull_sample().unwrap();
                println!("Got Video Sample: {:?}", sample);
                let buffer = sample.get_buffer().unwrap();
                println!("{:?}", buffer);
            
                let caps = sample.get_caps().expect("Sample without caps");
                let info = gst_video::VideoInfo::from_caps(&caps).expect("Failed to parse caps");
            
                // At this point, buffer is only a reference to an existing memory region somewhere.
                // When we want to access its content, we have to map it while requesting the required
                // mode of access (read, read/write).
                // This type of abstraction is necessary, because the buffer in question might not be
                // on the machine's main memory itself, but rather in the GPU's memory.
                // So mapping the buffer makes the underlying memory region accessible to us.
                // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
                let map = buffer.map_readable().unwrap();
            
                // We only want to have a single buffer and then have the pipeline terminate
                println!("Have video frame");
            
                let size = [info.width(), info.height()];
                let texture = CreateTexture::create(&mut mc_context.texture_context, 
                    Format::Rgba8, &map, size, &TextureSettings::new()).unwrap();

                /*
                let texture: G2dTexture = Texture::from_memory_alpha(
                    &mut mc_context.texture_context,
                    &map,
                    info.width(),
                    info.height(),
                    &TextureSettings::new()
                ).unwrap();*/
                let transform = transform.scale(0.1, 0.1);

                graphics::image(&texture, transform, gl);
            },
            WidgetType::Image { image  } => {
                let texture: G2dTexture = Texture::from_image(
                    &mut mc_context.texture_context,
                    &image,
                    &TextureSettings::new()
                ).unwrap();
                graphics::image(&texture, transform, gl);
            }
        }
    }
}
