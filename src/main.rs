extern crate gl;
extern crate gl_loader;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate graphics;
extern crate piston;
extern crate png;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use gl::types::*;
use png::Encoder;
use std::fs::File;
use std::io::{BufWriter};
use std::thread;
use std::process::{Command};

const WIDTH: u32 = 1920; 
const HEIGHT: u32 = 1080; 
const ALL: usize = 4 * WIDTH as usize * HEIGHT as usize;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(GREEN, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.rotation += 2.0 * args.dt;
    }
}

fn save_frame(frame_counter: &mut u32) {
    let mut pixel_data: Vec<u8> = vec![0; ALL];

    unsafe {
        gl::ReadBuffer(gl::FRONT);
        gl::ReadPixels(
            0,
            0,
            WIDTH as GLint,
            HEIGHT as GLint,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixel_data.as_mut_ptr() as *mut GLvoid,
        );
    }
    
    let filename = format!("images/frame_{:010}.png", frame_counter);
    thread::spawn(move ||{
        let file = File::create(filename).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = Encoder::new(w, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixel_data).unwrap();
    });

    println!("Saving frame: {}", frame_counter);
    *frame_counter += 1;

}

fn render_ffmpeg() {
    let ffmpeg = Command::new("ffmpeg")
        .arg("-y")
        .arg("-s")
        .arg(format!("{}x{}", WIDTH, HEIGHT))
        .arg("-r")
        .arg("60")
        .arg("-f")
        .arg("image2")
        .arg("-i")
        .arg("./images/frame_%10d.png")
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("rgba")
        .arg("-crf")
        .arg("20")
        .arg("-preset")
        .arg("fast")
        .arg("output.mp4")
        .output()
        .expect("Failed to start FFmpeg");

    if ffmpeg.status.success() {
        println!("Done!")
    } else {
        // Print error
        let err = String::from_utf8_lossy(&ffmpeg.stderr);
        println!("FFmpeg failed: {}", err);
    }
}

fn main() {
    let opengl = OpenGL::V3_3;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", (WIDTH, HEIGHT))
        .graphics_api(opengl)
        .resizable(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
    };

    let mut frame_counter = 0;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
            // render_ffmpeg();
            save_frame(&mut frame_counter);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
    render_ffmpeg();
}