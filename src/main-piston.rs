extern crate graphics;
extern crate piston_window;
extern crate find_folder;
extern crate fps_counter;

use gfx_device_gl::{CommandBuffer, NewTexture, TargetView};
use gfx_graphics::{Gfx2d, GfxGraphics};
use piston_window::*;
mod scene;
mod server;

use std::thread;
use std::sync::Arc;

extern crate gstreamer as gst;

use piston::input::RenderEvent;
use piston::window::WindowSettings;

use gfx::{Factory, format::{DepthStencil, Formatted, Srgba8}};
use gfx::memory::Typed;
use gfx_core::handle::Producer;
use gfx_core::memory::{Bind, Usage};
use gfx_core::{self as c, handle, state as s, format, pso, texture, command as com, buffer};
// use gfx_core::factory::Factory;
// use gfx_core::texture::{ SamplerInfo, FilterMethod, WrapMode, Size };
//use graphics::{ Context, Graphics, Transformed };

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
    

    let dim = [0, 0, texture::AaMode::Single];
    let color_format: gfx::format::Format = <Srgba8 as Formatted>::get_format();
    let depth_format: gfx::format::Format = <DepthStencil as Formatted>::get_format();
    let mut temp = handle::Manager::new();
    let color_tex = temp.make_texture(
        NewTexture::Surface(0),
        texture::Info {
            levels: 1,
            kind: texture::Kind::D2(100, 100, texture::AaMode::Single),
            format: color_format.0,
            bind: Bind::RENDER_TARGET | Bind::TRANSFER_SRC,
            usage: Usage::Data,
        },
    );
    let depth_tex = temp.make_texture(
        NewTexture::Surface(0),
        texture::Info {
            levels: 1,
            kind: texture::Kind::D2(100, 100, texture::AaMode::Single),
            format: depth_format.0,
            bind: Bind::DEPTH_STENCIL | Bind::TRANSFER_SRC,
            usage: Usage::Data,
        },
    );
    let m_color = temp.make_rtv(TargetView::Surface(0), &color_tex, dim);
    let m_ds = temp.make_dsv(TargetView::Surface(0), &depth_tex, dim);

    
    //let output_color = Typed::new(output_color);
    //let output_stencil = Typed::new(output_stencil);
    /*
    let ref mut g = GfxGraphics::new(
        &mut texture_context.encoder,
        &m_color.into(),
        &m_ds.into(),
        &mut window.g2d
    );*/


    //let fbo = Gfx2d::new(OpenGL::V2_1, &mut window.factory);

    // let size = window.draw_size();


    // let (texture_handle, shader_view, target) = window.factory.create_render_target(size.width as Size, size.height as Size)
    //     .expect("render target");

    // (gfx_core::handle::Texture<gfx_device_gl::Resources, _>, gfx_core::handle::ShaderResourceView<gfx_device_gl::Resources, _>, gfx_core::handle::RenderTargetView<gfx_device_gl::Resources, T>)
    // let sampler = window.factory.create_sampler(SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile));
    // let texture = G2dTexture {
    //     surface: texture_handle,
    //     sampler: sampler,
    //     view: shader_view,
    // };
    // let stencil = window.factory.create_depth_stencil_view_only(size.width as Size, size.height as Size)
    //     .expect("stencil");


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
