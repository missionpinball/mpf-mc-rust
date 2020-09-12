extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::Arc;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use tokio::prelude::*;
use tokio::time::delay_for;
use tokio::sync::{watch, Mutex};

use tonic::{transport::Server, Request, Response, Status};

use mpf::media_controller_server::{MediaController, MediaControllerServer};
use mpf::{SlideAddRequest, SlideAddResponse};

pub mod mpf {
    tonic::include_proto!("mpf"); // The string specified here must match the proto package name
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
    channel: std::sync::mpsc::Receiver<f64>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, args.window_size[0] / 4.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
        while let Ok(data) = self.channel.try_recv() {
            self.rotation += data;
        }
    }
}

#[derive(Debug)]
pub struct MyMediaController {
    channel: Arc<Mutex<std::sync::mpsc::Sender<f64>>>
}

#[tonic::async_trait]
impl MediaController for MyMediaController {

    async fn add_slide(&self, request: Request<SlideAddRequest>) ->
    Result<Response<SlideAddResponse>, Status> {
        let req = request.into_inner();
        println!("Slide {} Text: {}", req.slide_name, req.text);
       
        let channel = self.channel.lock().await;
        channel.send(10.0).unwrap();
        Ok(Response::new(SlideAddResponse{}))
    }
}

async fn serve(channel: std::sync::mpsc::Sender<f64>) {
    let addr = "[::1]:50051".parse().unwrap();
    let mc = MyMediaController{
        channel: Arc::new(Mutex::new(channel))
    };

    Server::builder()
        .add_service(MediaControllerServer::new(mc))
        .serve(addr)
        .await.unwrap();
}

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(serve(tx));
    });

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        channel: rx
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
