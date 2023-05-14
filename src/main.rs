extern crate gl;
extern crate gl_loader;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use gl::types::*;
use std::fs::{File, create_dir, remove_dir_all};
use std::io::Write;
use std::process::{Command, Stdio};

const WIDTH: u32 = 1920; 
const HEIGHT: u32 = 1080; 
const FRAMERATE: f32 = 60.0; 
const ALL: usize = 4 * WIDTH as usize * HEIGHT as usize;

pub struct App {
    gl: GlGraphics,
    pixel_data: Vec<u8>,
    rotation: f64,
    frame_counter: u32,
    index_file: File,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        self.draw(args);
        self.export();
        println!("Saving frame: {}", self.frame_counter);
        self.frame_counter += 1;
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.rotation += 2.0 * args.dt;
    }

    fn draw(&mut self, args: &RenderArgs) {
        use graphics::*;
        self.gl.clear_color([0.0, 0.0, 0.0, 1.0]);

        const RED: [f32; 4] = [1.0, 0.2, 0.2, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            rectangle(RED, square, transform, gl);
        });
    }

    fn export(&mut self) {

        // Render the frame to an ffmpeg video
        // let modulo: usize = frame_counter%4;
        unsafe {
            gl::ReadBuffer(gl::FRONT);
            gl::ReadPixels(
                0,
                0,
                WIDTH as GLint,
                HEIGHT as GLint,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.pixel_data.as_mut_ptr() as *mut GLvoid,
            );
        }
        let name = format!("temp/{:010}.mp4", self.frame_counter);
        let filename = name.as_str();
        let mut ffmpeg = Command::new("ffmpeg")
            .arg("-loglevel")
            .arg("16")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-s")
            .arg(format!("{}x{}", WIDTH, HEIGHT))
            .arg("-i")
            .arg("-")
            .arg("-vcodec")
            .arg("libx264")
            .arg("-crf")
            .arg("23")
            .arg("-t")
            .arg(format!("{}", 1.0/FRAMERATE))
            .arg("-y")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&self.pixel_data).unwrap();
        }
        writeln!(self.index_file, "file {}", filename).unwrap();
    }
}

fn concat_output() {
    Command::new("ffmpeg")
        .arg("-loglevel")
        .arg("0")
        .arg("-f")
        .arg("concat")
        .arg("-i")
        .arg("index.txt")
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg("output.mp4")
        .output()
        .unwrap();

    println!("\nFresh video generated!\n");
}

fn main() {
    remove_dir_all("temp").unwrap();
    create_dir("temp").unwrap();

    let opengl = OpenGL::V3_3;

    let mut window: Window = WindowSettings::new("spinning-square", (WIDTH, HEIGHT))
        .graphics_api(opengl)
        .resizable(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    let mut app = App {
        gl: GlGraphics::new(opengl),
        pixel_data: vec![0 ; ALL],
        rotation: 0.0,
        frame_counter: 0,
        index_file: File::create("index.txt").unwrap(),
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
    concat_output();
}