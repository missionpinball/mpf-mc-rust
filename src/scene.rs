use std::collections::HashMap;
use graphics::context::Context;
use graphics::text::Text;
use crate::graphics::Transformed;
use piston_window::*;
use std::sync::Mutex;
use std::sync::Arc;

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
    pub color: graphics::types::Color,
    pub widget: WidgetType
}

impl Widget {
    fn draw(&self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        // TODO: add graphics_buffer and only render when something changed

        self.render(c, gl, mc_context)
    }

    fn render(&self, c: &Context, gl: &mut G2d, mc_context: &mut super::MediaControllerContext) {
        let transform = c.transform.trans(self.x, self.y);

        match &self.widget {
            WidgetType::Text { text } => {
                Text::new_color(self.color, 32).draw(
                                text,
                                &mut mc_context.glyphs,
                                &c.draw_state,
                                transform, gl
                            ).unwrap();
            }
            WidgetType::Rectacle { width, height } => {
                rectangle(self.color, [0.0, 0.0, *width, *height], transform, gl);
            },
            WidgetType::Video {  } => {
                
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

#[derive(Debug)]
pub enum WidgetType {
    Text {
        text: String
    },
    Rectacle {
        width: f64,
        height: f64
    },
    Image {
        image: image::RgbaImage
    },
    Video {

    }
}