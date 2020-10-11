extern crate ggez;

use ggez::event;
use ggez::graphics;
use ggez::timer;
use ggez::{Context, GameResult};
use gst::prelude::Cast;
use gst_gl::GLContextExt;
use std::env;
use std::path;

use ggez::nalgebra::Point2;

mod scene;
mod server;

use std::thread;
use std::sync::Arc;

extern crate gstreamer as gst;
extern crate gstreamer_gl as gst_gl;

use glutin::os::{ContextTraitExt, unix::WindowExt, unix::RawHandle};

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

fn create_gst_gl_context(ctx: &mut Context) -> gst_gl::GLContext {
    let windowed_context = graphics::window_raw(ctx);
    let raw_handle = unsafe { windowed_context.raw_handle() };
    let inner_window = windowed_context.window();
    let shared_context: gst_gl::GLContext;
    let api: gst_gl::GLAPI = gst_gl::GLAPI::OPENGL3;

    let (gl_context, gl_display, platform) = match raw_handle
    {
        RawHandle::Egl(egl_context) => {

            let gl_display = if let Some(display) =
                unsafe { windowed_context.get_egl_display() }
            {
                unsafe { gst_gl::GLDisplayEGL::with_egl_display(display as usize) }.unwrap()
            } else {
                panic!("EGL context without EGL display");
            };

            let gl_display = if let Some(display) = inner_window.get_wayland_display() {
                unsafe { gst_gl::GLDisplayWayland::with_display(display as usize) }.unwrap()
            } else {
                panic!("Wayland window without Wayland display");
            };

            (
                egl_context as usize,
                gl_display.upcast::<gst_gl::GLDisplay>(),
                gst_gl::GLPlatform::EGL,
            )
        }
        RawHandle::Glx(glx_context) => {
            let gl_display = if let Some(display) = inner_window.get_xlib_display() {
                unsafe { gst_gl::GLDisplayX11::with_display(display as usize) }.unwrap()
            } else {
                panic!("X11 window without X Display");
            };

            (
                glx_context as usize,
                gl_display.upcast::<gst_gl::GLDisplay>(),
                gst_gl::GLPlatform::GLX,
            )
        }
        #[allow(unreachable_patterns)]
        handler => panic!("Unsupported platform: {:?}.", handler),
    };

    shared_context = unsafe { gst_gl::GLContext::new_wrapped(&gl_display, gl_context, platform, api) }.unwrap();

    shared_context
        .activate(true)
        .expect("Couldn't activate wrapped GL context");

    shared_context.fill_info().unwrap();

    shared_context
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

    let cb = ggez::ContextBuilder::new("MPF Media Controller", "jab").add_resource_path(resource_dir);
    let cb = cb.window_setup(ggez::conf::WindowSetup::default().title("MPF Media Controller"));
    let (ctx, events_loop) = &mut cb.build()?;
    let gst_gl_context = create_gst_gl_context(ctx);

    let scene_server = scene.clone();
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(server::serve(scene_server, gst_gl_context));
    });

    let state = &mut MainState::new(ctx, scene)?;
    event::run(ctx, events_loop, state)
}
