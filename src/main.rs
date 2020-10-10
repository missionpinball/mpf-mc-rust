extern crate ggez;
extern crate cgmath;

use ggez::event;
use ggez::graphics;
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::path;

type Point2 = cgmath::Point2<f32>;
type Vector2 = cgmath::Vector2<f32>;

mod scene;
mod server;

use std::thread;
use std::sync::Arc;

extern crate gstreamer as gst;


struct MainState {
    scene: Arc<scene::Scene>
}

impl MainState {
    fn new(ctx: &mut Context, scene: Arc<scene::Scene>) -> GameResult<MainState> {
        let s = MainState {
            scene
        };
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
        current_scene.lock().unwrap().update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        let origin = Point2::new(0.0, 0.0);

        let current_scene = self.scene.current_slide.lock().unwrap();
        current_scene.lock().unwrap().draw(ctx, origin);
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

    let scene_server = scene.clone();
    let asset_path = resource_dir.clone();    
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::serve(scene_server, asset_path));
    });

    let cb = ggez::ContextBuilder::new("MPF Media Controller", "jab").add_resource_path(resource_dir);
    let cb = cb.window_setup(ggez::conf::WindowSetup::default().title("MPF Media Controller"));
    let (ctx, events_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx, scene)?;
    event::run(ctx, events_loop, state)
}
