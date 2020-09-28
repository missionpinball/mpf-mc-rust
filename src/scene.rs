use std::collections::HashMap;
use graphics::context::Context;
use graphics::math::Matrix2d;
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

pub struct Slide {
    pub widgets: Vec<Widget>

}

impl Slide {
    pub fn render(&mut self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        for widget in &mut self.widgets {
            widget.draw(c, gl, mc_context);
        }
    }
}

pub struct Widget {
    pub x: f64,
    pub y: f64,
    pub z: u32,
    pub id: u32,
    pub widget: WidgetType,
    pub render_state: RenderState
}

pub enum RenderState {
    NotRenderedYet,
    TextureDirty {
        texture: G2dTexture
    },
    TextureRendered {
        texture: G2dTexture
    }
    
}

pub enum WidgetType {
    Label {
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
    ImageSprite {
        image: image::RgbaImage
        // TODO: add sprite stuff
    },
    ImageAnimated {
        image: image::RgbaImage
        // TODO add animation stuff
    },
    Video {
        pipeline: gst::Pipeline,
        video_sink: gst_app::AppSink
    },
    Line {
        line: graphics::line::Line
    }
}

impl Widget {
    fn draw(&mut self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        let transform = c.transform.trans(self.x, self.y);
        match &self.render_state {
            RenderState::NotRenderedYet => {
                self.render(c, gl, transform, mc_context)
            }
            RenderState::TextureDirty { texture } => {
                self.render(c, gl, transform, mc_context)
            }
            RenderState::TextureRendered { texture } => {
                graphics::image(texture, transform, gl);
            }
        }
    }

    fn render(&mut self, c: &Context, gl: &mut G2d, transform: Matrix2d, mc_context: &mut super::MediaControllerContext) {
        

        match &mut self.widget {
            WidgetType::Label { text, color} => {
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
            WidgetType::Video { video_sink, .. } => {
                if video_sink.is_eos() {
                    if let RenderState::TextureDirty{ texture} = &mut self.render_state {
                        // Take the last rendered frame and keep that for this video
                        graphics::image(texture, transform, gl);
                        self.render_state = RenderState::TextureRendered {                            
                            texture: texture.clone()
                        };
                    }
                    return
                }
                
                let sample = video_sink.pull_sample().unwrap();
                let buffer = sample.get_buffer().unwrap();
            
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
            
                let size = [info.width(), info.height()];

                // TODO: move do this into gstreamer
                let transform = transform.scale(0.1, 0.1);
                
                /*if let RenderState::TextureDirty{ texture} = &mut self.render_state {
                    UpdateTexture::update(&mut *texture, &mut mc_context.texture_context, 
                        Format::Rgba8, &map, [0, 0], size).unwrap();
                    graphics::image(texture, transform, gl);
                } else {*/
                    let texture = CreateTexture::create(&mut mc_context.texture_context, 
                        Format::Rgba8, &map, size, &TextureSettings::new()).unwrap();
                    graphics::image(&texture, transform, gl);
                    self.render_state = RenderState::TextureDirty{texture};
                //}
            },
            WidgetType::Image { image  } => {                
                let texture: G2dTexture = Texture::from_image(
                    &mut mc_context.texture_context,
                    &image,
                    &TextureSettings::new()
                ).unwrap();                
                graphics::image(&texture, transform, gl);
                self.render_state = RenderState::TextureRendered{texture};
            }
            WidgetType::ImageSprite { image } => {

            }
            WidgetType::ImageAnimated { image } => {

            }
            WidgetType::Line { line } => {

            }
        }
    }
}
