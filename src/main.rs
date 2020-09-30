extern crate graphics;
extern crate piston_window;
extern crate find_folder;
extern crate fps_counter;

use piston_window::*;
mod scene;
mod server;

use std::thread;
use std::sync::Arc;

extern crate gstreamer as gst;

use piston::input::RenderEvent;
use piston::window::WindowSettings;

pub struct MediaControllerContext {
    glyphs: Glyphs,
    texture_context: G2dTextureContext
}

fn main() {
    gst::init().unwrap();

    let scene = Arc::new(scene::Scene::new());

    let render_size = [400.0, 200.0];
    let mut window: PistonWindow = WindowSettings::new(
        "MPF Media Controller",
        render_size
    )
    .exit_on_esc(true)
    //.opengl(OpenGL::V2_1) // Set a different OpenGl version
    .build()
    .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();

    let scene_server = scene.clone();
    let asset_path = assets.clone();    
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::serve(scene_server, asset_path));
    });

    let texture_context = window.create_texture_context();
    let mut mc_context = MediaControllerContext {
        glyphs,
        texture_context
    };

    let mut fps_counter = fps_counter::FPSCounter::new();

    //window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g, device| {
                clear([0.0, 0.0, 0.0, 1.0], g);
                let c = c.scale(args.window_size[0] / render_size[0], args.window_size[1] / render_size[1]);
                let current_scene = scene.current_slide.lock().unwrap();
                current_scene.lock().unwrap().render(&c, g, &mut mc_context);

                // Render FPS for now
                let fps = fps_counter.tick();
                let transform = c.transform.trans(200.0, 30.0);
                Text::new_color([1.0, 1.0, 1.0, 1.0], 32).draw(
                    &format!("FPS: {}", fps),
                    &mut mc_context.glyphs,
                    &c.draw_state,
                    transform, g
                ).unwrap();

                // Update glyphs before rendering.
                mc_context.glyphs.factory.encoder.flush(device);
            });
        }
    }
}
