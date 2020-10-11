use std::{collections::HashMap};
use std::sync::Mutex;
use std::sync::Arc;

use ggez::{Context, graphics::DrawParam};
use ggez::graphics;
use ggez::nalgebra::Point2;

use arc_swap::ArcSwapOption;

extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_video as gst_video;
//use self::gst_video::prelude::*;

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
    pub fn draw(&mut self, ctx: &mut Context, origin: Point2<f32>) {
        for widget in &mut self.widgets {
            widget.draw(ctx, origin);
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        for widget in &mut self.widgets {
            if let UpdateState::NeedsUpdate = widget.update_state {
                widget.update(ctx);
            }
        }
    }
}

pub struct Widget {
    pub x: f32,
    pub y: f32,
    pub z: u32,
    pub id: u32,
    pub widget: WidgetType,
    pub render_state: RenderState,
    pub update_state: UpdateState,
}

pub enum RenderState {
    ImageTextureRendered {
        image_texture: graphics::Image
    },
    CanvasRendered {
        canvas: graphics::Canvas
    },
    NoContent
}

pub enum UpdateState {
    NeedsUpdate,
    Clean,
}

pub enum WidgetType {
    Label {
        text: String,
        color: graphics::Color,
        font: Option<graphics::Font>,
        font_size: u32
    },
    Rectacle {
        width: f64,
        height: f64,
        color: graphics::Color,
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
        video_sink: gst_app::AppSink,
        video_memory: Arc<ArcSwapOption<gst::Sample>>
    },
    Line {
        x1: f64,
        x2: f64,
        y1: f64,
        y2: f64,
    }
}

impl Widget {
    fn draw(&mut self, ctx: &mut Context, origin: Point2<f32>) {
        // TODO: fix transform here
        let draw_param = DrawParam::default().dest(Point2::new(self.x, self.y));
        match &self.render_state {
            RenderState::ImageTextureRendered { image_texture} => {
                graphics::draw(ctx, image_texture, draw_param).unwrap();
            }
            RenderState::CanvasRendered { canvas } => {
                graphics::draw(ctx, canvas, draw_param).unwrap();
            }
            RenderState::NoContent => {
                // Do nothing
            }
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        match &mut self.widget {
            WidgetType::Label { text, color, font_size, font} => {
                let loaded_font;
                match font.as_mut() {
                    None => {
                        loaded_font = graphics::Font::new(ctx, "/DejaVuSerif.ttf").unwrap();
                        *font = Some(loaded_font);
                    },
                    Some(v) => {
                        loaded_font = *v;
                    }
                }
                    
                let text = graphics::Text::new((text.to_string(), loaded_font, *font_size as f32));
                let (width, height) = text.dimensions(ctx);
                let canvas = graphics::Canvas::new(ctx, width as u16, height as u16, ggez::conf::NumSamples::One, graphics::get_window_color_format(ctx)).unwrap();

                graphics::set_canvas(ctx, Some(&canvas));
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width as f32, height as f32)).unwrap();
                graphics::clear(ctx, graphics::Color::from((0, 0, 0, 0)));
                graphics::draw(ctx, &text, graphics::DrawParam::new().color(*color)).unwrap();
                graphics::set_canvas(ctx, None);

                let (w, h) = graphics::drawable_size(ctx);
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, w, h)).unwrap();

                self.render_state = RenderState::CanvasRendered{canvas};
                self.update_state = UpdateState::Clean;
            }
            WidgetType::Rectacle { width, height, color } => {
                let rect_dimensions = graphics::Rect::new(0.0, 0.0, *width as f32, *height as f32);
                let rect = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect_dimensions, *color).unwrap();
                let canvas = graphics::Canvas::new(ctx, *width as u16, *height as u16, ggez::conf::NumSamples::One, graphics::get_window_color_format(ctx)).unwrap();

                graphics::set_canvas(ctx, Some(&canvas));
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, *width as f32, *height as f32)).unwrap();
                graphics::clear(ctx, graphics::Color::from((0, 0, 0, 0)));
                graphics::draw(ctx, &rect, graphics::DrawParam::new().color(*color)).unwrap();
                graphics::set_canvas(ctx, None);

                let (w, h) = graphics::drawable_size(ctx);
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, w, h)).unwrap();

                self.render_state = RenderState::CanvasRendered{canvas};
                self.update_state = UpdateState::Clean;

            },
            WidgetType::Video { video_sink, video_memory, ..} => {
                if video_sink.is_eos() {
                    self.update_state = UpdateState::Clean;
                    return
                }
                let frame = video_memory.as_ref().load();
                if let Some(frame) = frame.as_ref() {
                    let buffer = frame.get_buffer().unwrap();
                    let caps = frame.get_caps().expect("Sample without caps");
                    let info = gst_video::VideoInfo::from_caps(&caps).expect("Failed to parse caps");
                    let image = buffer.map_readable().unwrap();
    
                    // TODO: move do this into gstreamer
                    //let transform = transform.scale(0.1, 0.1);
                
                    let image_texture = graphics::Image::from_rgba8(ctx, info.width() as u16, info.height() as u16, &image).unwrap();
                    self.render_state = RenderState::ImageTextureRendered{image_texture};
                }
            },
            WidgetType::Image { image  } => {
                let (width, height) = image.dimensions();
                let image_texture = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &image).unwrap();
                self.render_state = RenderState::ImageTextureRendered{image_texture};
                self.update_state = UpdateState::Clean;
            }
            WidgetType::ImageSprite { image } => {

            }
            WidgetType::ImageAnimated { image } => {

            }
            WidgetType::Line { x1, x2, y1, y2 } => {
                /*
                let rect_dimensions = graphics::Rect::new(0.0, 0.0, *width as f32, *height as f32);
                let rect = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect_dimensions, *color).unwrap();
                let canvas = graphics::Canvas::new(ctx, *width as u16, *height as u16, ggez::conf::NumSamples::One).unwrap();

                graphics::set_canvas(ctx, Some(&canvas));
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, *width as f32, *height as f32)).unwrap();
                graphics::clear(ctx, graphics::Color::from((0, 0, 0, 0)));
                graphics::draw(ctx, &rect, graphics::DrawParam::new().color(*color)).unwrap();
                graphics::set_canvas(ctx, None);

                let (w, h) = graphics::drawable_size(ctx);
                graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, w, h)).unwrap();

                self.render_state = RenderState::CanvasRendered{canvas};
                self.update_state = UpdateState::Clean;
                */
            }
        }
    }
}
