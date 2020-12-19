extern crate ggez;

use ggez::event;
use ggez::graphics;
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::path;

use ggez::mint::Point2;

mod scene;
mod server;

#[cfg(feature = "gst-gl-video")]
mod gst_gl_video;

use std::sync::Arc;
use std::thread;

extern crate gstreamer as gst;

struct MainState {
    scene: Arc<scene::Scene>,
}

impl MainState {
    fn new(_: &mut Context, scene: Arc<scene::Scene>) -> GameResult<MainState> {
        let s = MainState { scene };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::delta(ctx));
            println!("Average FPS: {}", timer::fps(ctx));
        }

        let current_scene = self.scene.current_slide.lock().unwrap();
        current_scene.lock().unwrap().update(ctx)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        let origin = Point2{x: 0.0, y: 0.0};

        let current_scene = self.scene.current_slide.lock().unwrap();
        current_scene.lock().unwrap().draw(ctx, origin)?;
        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    gst::init().unwrap();

    let scene = Arc::new(scene::Scene::new());

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb =
        ggez::ContextBuilder::new("MPF Media Controller", "jab").add_resource_path(resource_dir);
    let cb = cb.window_setup(ggez::conf::WindowSetup::default().title("MPF Media Controller"));
    let (mut ctx, events_loop) = cb.build()?;

    #[cfg(feature = "gst-gl-video")]
    let (gst_gl_context, gl_display) = gst_gl_video::create_gst_gl_context(ctx);

    let scene_server = scene.clone();
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::serve(
            scene_server,
            #[cfg(feature = "gst-gl-video")]
            gst_gl_context,
            #[cfg(feature = "gst-gl-video")]
            gl_display,
        ));
    });

    let state = MainState::new(&mut ctx, scene)?;
    event::run(ctx, events_loop, state)
}
