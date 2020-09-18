extern crate graphics;
extern crate piston_window;
extern crate find_folder;

use piston_window::*;
mod scene;
mod server;

use std::thread;
use std::sync::Arc;

use piston::input::RenderEvent;
use piston::window::WindowSettings;

fn main() {
    let scene = Arc::new(scene::Scene::new());

    let scene_server = scene.clone();
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::serve(scene_server));
    });

    let render_size = [400.0, 200.0];
    let mut window: PistonWindow = WindowSettings::new(
        "piston: hello_world",
        render_size
    )
    .exit_on_esc(true)
    //.opengl(OpenGL::V2_1) // Set a different OpenGl version
    .build()
    .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();

    //window.set_lazy(true);
    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g, device| {
                //let transform = c.transform.trans(10.0, 100.0);

                clear([0.0, 0.0, 0.0, 1.0], g);
                let c = c.scale(args.window_size[0] / render_size[0], args.window_size[1] / render_size[1]);
                let current_scene = scene.current_slide.lock().unwrap();
                current_scene.lock().unwrap().render(&c, g, &mut glyphs);

                // Update glyphs before rendering.
                glyphs.factory.encoder.flush(device);
            });
        }
    }
}
