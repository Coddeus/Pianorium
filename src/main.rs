extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate gl;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use image::{ImageBuffer, DynamicImage, ImageFormat};
use gl::types::*;
use std::thread;

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
    }
}

fn save_frame(size: (u32, u32), frame_counter: &mut u32) {
    let (width, height) = size;
    let mut pixel_data: Vec<u8> = vec![0; 4 * width as usize * height as usize];

    unsafe {
        gl::ReadBuffer(gl::FRONT);
        gl::ReadPixels(
            0,
            0,
            width as GLint,
            height as GLint,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixel_data.as_mut_ptr() as *mut GLvoid,
        );
    }
    *frame_counter += 1;
    let filename = format!("images/frame_{:010}.png", frame_counter);
    thread::spawn(move || {
        let dynamic_image = DynamicImage::ImageRgba8(ImageBuffer::from_raw(width, height, pixel_data).unwrap());
        dynamic_image.save_with_format(filename, ImageFormat::Jpeg).unwrap();
    });
    println!("Saving frame: {}", frame_counter);

}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_3;
    // (1920, 1080) => 11.75fps, (960, 540) => 45fps, (100, 100) => 60fps
    let size = (100, 100); 

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", size)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

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
            thread::spawn(move || {
                save_frame(size, &mut frame_counter);
                // println!("Saved frame: {}", frame_counter);
                // frame_counter+=1;
            });
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}