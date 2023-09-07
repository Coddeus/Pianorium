use crate::gl;
use egui_sdl2_gl::egui::{color::Hsva, color_picker::Alpha};

use crate::{create_program, Program, Uniform};

// pub duration: f64,
// pub tempo: f32,
// pub border_radius: u8,
// pub bgimg_file: &'static str,
// pub vertical_img_divide: u8,
/// Rendering parameters
pub struct Parameters {
    pub bytes: usize,
    pub max_cores: usize,
    pub max_time: f32,
    pub index_file: String,

    pub width: usize,
    pub height: usize,
    pub cores: usize,
    pub samples: u8,
    pub framerate: f32,
    pub latest_gravity: f32,
    pub gravity: f32,
    pub preview_speed: f32,
    pub midi_file: String,
    pub mp4_file: String,
    pub png_file: String,
    pub clear_dir: bool,
    pub bg: Hsva,
    pub alpha: Alpha,

    pub program: Program,

    pub time: f32,
    pub octave_line: f32,
    pub octave_line_color: Hsva,
    pub note_left: Hsva,
    pub note_right: Hsva,
    pub note_top: Hsva,
    pub note_bottom: Hsva,
    pub note_time: Hsva,
    pub particle_left: Hsva,
    pub particle_right: Hsva,
    pub particle_top: Hsva,
    pub particle_bottom: Hsva,
    pub particle_time: Hsva,

    pub u_time: Uniform,
    pub u_resolution: Uniform,
    pub u_octave_line_color: Uniform,
    pub u_note_left: Uniform,
    pub u_note_right: Uniform,
    pub u_note_top: Uniform,
    pub u_note_bottom: Uniform,
    pub u_note_time: Uniform,
    pub u_particle_left: Uniform,
    pub u_particle_right: Uniform,
    pub u_particle_top: Uniform,
    pub u_particle_bottom: Uniform,
    pub u_particle_time: Uniform,
}

impl Default for Parameters {
    fn default() -> Self {
        let width: usize = 800;
        let height: usize = 600;
        let bytes: usize = width * height * 4;
        let max_cores: usize = num_cpus::get();
        let cores: usize = max_cores;
        let samples: u8 = 11;
        let framerate: f32 = 60.0;
        let time: f32 = 0.0;
        let max_time: f32 = 0.0;
        let latest_gravity: f32 = 1.0;
        let gravity: f32 = 1.0;
        let preview_speed: f32 = 1.0;
        let midi_file: String = "test.mid".to_owned();
        let mp4_file: String = "output.mp4".to_owned();
        let png_file: String = "output.png".to_owned();
        let clear_dir: bool = false;
        let index_file: String = "pianorium_index.txt".to_owned();
        let octave_line: f32 = 0.0;
        let octave_line_color: Hsva = Hsva {
            h: 0.2,
            s: 0.1,
            v: 1.0,
            a: 1.0,
        };
        let bg: Hsva = Hsva {
            h: 0.0,
            s: 0.0,
            v: 0.1,
            a: 1.0,
        };
        let alpha: Alpha = Alpha::Opaque;

        let program: Program = create_program().unwrap();
        program.set_used();

        let note_left: Hsva = Hsva {
            h: 0.75,
            s: 0.7,
            v: 1.0,
            a: 1.0,
        };
        let note_right: Hsva = Hsva {
            h: 0.95,
            s: 0.7,
            v: 1.0,
            a: 1.0,
        };
        let note_top: Hsva = Hsva {
            h: 0.70,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        };
        let note_bottom: Hsva = Hsva {
            h: 0.96,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        };
        let note_time: Hsva = Hsva {
            h: 0.7,
            s: 0.5,
            v: 0.5,
            a: 1.0,
        };
        let particle_left: Hsva = Hsva {
            h: 0.66,
            s: 0.31,
            v: 1.0,
            a: 1.0,
        };
        let particle_right: Hsva = Hsva {
            h: 0.0,
            s: 0.31,
            v: 1.0,
            a: 1.0,
        };
        let particle_top: Hsva = Hsva {
            h: 0.0,
            s: 0.0,
            v: 0.0,
            a: 1.0,
        };
        let particle_bottom: Hsva = Hsva {
            h: 0.0,
            s: 0.0,
            v: 1.0,
            a: 1.0,
        };
        let particle_time: Hsva = Hsva {
            h: 0.75,
            s: 0.5,
            v: 0.5,
            a: 1.0,
        };

        let u_time: Uniform = Uniform::new(program.id, "u_time").unwrap();
        let u_resolution: Uniform = Uniform::new(program.id, "u_resolution").unwrap();
        let u_octave_line_color: Uniform = Uniform::new(program.id, "u_octave_line_color").unwrap();
        let u_note_left: Uniform = Uniform::new(program.id, "u_note_left").unwrap();
        let u_note_right: Uniform = Uniform::new(program.id, "u_note_right").unwrap();
        let u_note_top: Uniform = Uniform::new(program.id, "u_note_top").unwrap();
        let u_note_bottom: Uniform = Uniform::new(program.id, "u_note_bottom").unwrap();
        let u_note_time: Uniform = Uniform::new(program.id, "u_note_time").unwrap();
        let u_particle_left: Uniform = Uniform::new(program.id, "u_particle_left").unwrap();
        let u_particle_right: Uniform = Uniform::new(program.id, "u_particle_right").unwrap();
        let u_particle_top: Uniform = Uniform::new(program.id, "u_particle_top").unwrap();
        let u_particle_bottom: Uniform = Uniform::new(program.id, "u_particle_bottom").unwrap();
        let u_particle_time: Uniform = Uniform::new(program.id, "u_particle_time").unwrap();

        unsafe {
            gl::Uniform1f(u_time.id, 0.0);
            gl::Uniform2f(u_resolution.id, width as f32, height as f32);
            gl::Uniform3f(
                u_octave_line_color.id,
                octave_line_color.to_rgb()[0],
                octave_line_color.to_rgb()[1],
                octave_line_color.to_rgb()[2],
            );
            gl::Uniform3f(
                u_note_left.id,
                note_left.to_rgb()[0],
                note_left.to_rgb()[1],
                note_left.to_rgb()[2],
            );
            gl::Uniform3f(
                u_note_right.id,
                note_right.to_rgb()[0],
                note_right.to_rgb()[1],
                note_right.to_rgb()[2],
            );
            gl::Uniform3f(
                u_note_top.id,
                note_top.to_rgb()[0],
                note_top.to_rgb()[1],
                note_top.to_rgb()[2],
            );
            gl::Uniform3f(
                u_note_bottom.id,
                note_bottom.to_rgb()[0],
                note_bottom.to_rgb()[1],
                note_bottom.to_rgb()[2],
            );
            gl::Uniform3f(
                u_note_time.id,
                note_time.to_rgb()[0],
                note_time.to_rgb()[1],
                note_time.to_rgb()[2],
            );
            gl::Uniform3f(
                u_particle_left.id,
                particle_left.to_rgb()[0],
                particle_left.to_rgb()[1],
                particle_left.to_rgb()[2],
            );
            gl::Uniform3f(
                u_particle_right.id,
                particle_right.to_rgb()[0],
                particle_right.to_rgb()[1],
                particle_right.to_rgb()[2],
            );
            gl::Uniform3f(
                u_particle_top.id,
                particle_top.to_rgb()[0],
                particle_top.to_rgb()[1],
                particle_top.to_rgb()[2],
            );
            gl::Uniform3f(
                u_particle_bottom.id,
                particle_bottom.to_rgb()[0],
                particle_bottom.to_rgb()[1],
                particle_bottom.to_rgb()[2],
            );
            gl::Uniform3f(
                u_particle_time.id,
                particle_time.to_rgb()[0],
                particle_time.to_rgb()[1],
                particle_time.to_rgb()[2],
            );
        }
        Parameters {
            bytes,
            max_cores,
            max_time,
            index_file,

            width,
            height,
            cores,
            samples,
            framerate,
            preview_speed,
            latest_gravity,
            gravity,
            midi_file,
            mp4_file,
            png_file,
            clear_dir,
            bg,
            octave_line,
            octave_line_color,
            alpha,

            program,

            time,

            note_left,
            note_right,
            note_top,
            note_bottom,
            note_time,
            particle_left,
            particle_right,
            particle_top,
            particle_bottom,
            particle_time,

            u_time,
            u_resolution,
            u_octave_line_color,
            u_note_left,
            u_note_right,
            u_note_top,
            u_note_bottom,
            u_note_time,
            u_particle_left,
            u_particle_right,
            u_particle_top,
            u_particle_bottom,
            u_particle_time,
        }
    }
}