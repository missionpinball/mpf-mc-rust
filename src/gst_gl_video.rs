use ggez::graphics;
use ggez::Context;
use glutin::os::{unix::RawHandle, unix::WindowExt, ContextTraitExt};
extern crate gstreamer_gl as gst_gl;
use gst::prelude::Cast;
use gst_gl::GLContextExt;

pub(crate) fn create_gst_gl_context(ctx: &mut Context) -> (gst_gl::GLContext, gst_gl::GLDisplay) {
    let windowed_context = graphics::window_raw(ctx);
    let raw_handle = unsafe { windowed_context.raw_handle() };
    let inner_window = windowed_context.window();
    let shared_context: gst_gl::GLContext;
    let api: gst_gl::GLAPI = gst_gl::GLAPI::OPENGL3;

    let (gl_context, gl_display, platform) = match raw_handle {
        RawHandle::Egl(egl_context) => {
            // TODO: readd cfgs here to make sure we only test one of them
            let gl_display = if let Some(display) = unsafe { windowed_context.get_egl_display() } {
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

    shared_context =
        unsafe { gst_gl::GLContext::new_wrapped(&gl_display, gl_context, platform, api) }.unwrap();

    shared_context
        .activate(true)
        .expect("Couldn't activate wrapped GL context");

    shared_context.fill_info().unwrap();

    (shared_context, gl_display)
}
