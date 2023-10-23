use std::path::PathBuf;

use crate::gl;
use egui_sdl2_gl::{
    egui::{color::Hsva, color_picker::Alpha},
    gl::types::GLint,
};

use crate::{create_program, Program, Uniform};

// pub duration: f64,
// pub tempo: f32,
// pub border_radius: u8,
// pub bgimg_file: &'static str,
// pub vertical_img_divide: u8,
/// The rendering parameters
pub struct Parameters {
    // CPU stuff
    pub bytes: usize,
    pub max_cores: usize,
    pub max_time: f32,

    pub width: usize,
    pub height: usize,
    pub cores: usize,
    pub max_samples: u8,
    pub samples: u8,
    pub framerate: i32,
    pub latest_gravity: f32,
    pub gravity: f32,
    pub ol_show: bool,
    pub ol_width: f32,
    pub note_show: bool,
    pub particle_show: bool,
    pub particle_density: u32,
    pub preview_speed: f32,
    pub midi_file: PathBuf,
    pub mkv_file: String,
    pub png_file: PathBuf,
    pub x264_preset: String,
    pub clear_dir: bool,
    pub bg: Hsva,
    pub alpha: Alpha,

    // GPU stuff
    pub program: Program,

    pub time: f32,
    pub vflip: bool,
    pub ol_color: Hsva,
    pub note_left: Hsva,
    pub note_right: Hsva,
    pub note_top: Hsva,
    pub note_bottom: Hsva,
    pub particle_left: Hsva,
    pub particle_right: Hsva,
    pub particle_top: Hsva,
    pub particle_bottom: Hsva,
    pub particle_transparency: f32,

    // pub u_time: Uniform,
    // pub u_resolution: Uniform,
    pub u_vflip: Uniform,
    pub u_ol_color: Uniform,
    pub u_note_left: Uniform,
    pub u_note_right: Uniform,
    pub u_note_top: Uniform,
    pub u_note_bottom: Uniform,
    pub u_particle_left: Uniform,
    pub u_particle_right: Uniform,
    pub u_particle_top: Uniform,
    pub u_particle_bottom: Uniform,
    pub u_particle_transparency: Uniform,
}

impl Default for Parameters {
    fn default() -> Self {
        let width: usize = 1920;
        let height: usize = 1080;
        let bytes: usize = width * height * 3;
        let max_cores: usize = num_cpus::get();
        let cores: usize = max_cores;
        let mut max_samples: GLint = 0;
        let mut max_tex_samples: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_SAMPLES, &mut max_samples);
            gl::GetIntegerv(gl::MAX_COLOR_TEXTURE_SAMPLES, &mut max_tex_samples);
        }
        let max_samples: u8 = max_samples as u8;
        let samples: u8 = max_samples.max(max_tex_samples as u8);
        let framerate: i32 = 60;
        let time: f32 = 0.0;
        let max_time: f32 = 0.0;
        let vflip: bool = false;
        let latest_gravity: f32 = 1.0;
        let gravity: f32 = 1.0;
        let ol_show: bool = true;
        let ol_width: f32 = 0.0001;
        let note_show: bool = true;
        let particle_show: bool = true;
        let particle_density: u32 = 3000;
        let preview_speed: f32 = 1.0;
        let midi_file: PathBuf = PathBuf::from("input.mid");
        let mkv_file: String = String::from("output.avi");
        let png_file: PathBuf = PathBuf::from("output.png");
        let x264_preset: String = "medium".to_owned();
        let clear_dir: bool = false;
        let ol_color: Hsva = Hsva {
            h: 0.2,
            s: 0.3,
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
        let particle_transparency: f32 = 0.5;

        // let u_time: Uniform = Uniform::new(program.id, "u_time").unwrap();
        let u_vflip: Uniform = Uniform::new(program.id, "u_vflip").unwrap();
        // let u_resolution: Uniform = Uniform::new(program.id, "u_resolution").unwrap();
        let u_ol_color: Uniform = Uniform::new(program.id, "u_ol_color").unwrap();
        let u_note_left: Uniform = Uniform::new(program.id, "u_note_left").unwrap();
        let u_note_right: Uniform = Uniform::new(program.id, "u_note_right").unwrap();
        let u_note_top: Uniform = Uniform::new(program.id, "u_note_top").unwrap();
        let u_note_bottom: Uniform = Uniform::new(program.id, "u_note_bottom").unwrap();
        let u_particle_left: Uniform = Uniform::new(program.id, "u_particle_left").unwrap();
        let u_particle_right: Uniform = Uniform::new(program.id, "u_particle_right").unwrap();
        let u_particle_top: Uniform = Uniform::new(program.id, "u_particle_top").unwrap();
        let u_particle_bottom: Uniform = Uniform::new(program.id, "u_particle_bottom").unwrap();
        let u_particle_transparency: Uniform =
            Uniform::new(program.id, "u_particle_transparency").unwrap();

        unsafe {
            // gl::Uniform1f(u_time.id, 0.0);
            gl::Uniform1i(u_vflip.id, vflip as i32);
            // gl::Uniform2f(u_resolution.id, width as f32, height as f32);
            gl::Uniform3f(
                u_ol_color.id,
                ol_color.to_rgb()[0],
                ol_color.to_rgb()[1],
                ol_color.to_rgb()[2],
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
            gl::Uniform1f(u_particle_transparency.id, particle_transparency);
        }
        Parameters {
            bytes,
            max_cores,
            max_time,

            width,
            height,
            cores,
            max_samples,
            samples,
            framerate,
            preview_speed,
            latest_gravity,
            gravity,
            ol_show,
            ol_width,
            note_show,
            particle_show,
            particle_density,
            midi_file,
            mkv_file,
            png_file,
            x264_preset,
            clear_dir,
            bg,
            alpha,

            program,

            time,
            vflip,
            ol_color,
            note_left,
            note_right,
            note_top,
            note_bottom,
            particle_left,
            particle_right,
            particle_top,
            particle_bottom,
            particle_transparency,

            // u_time,
            u_vflip,
            // u_resolution,
            u_ol_color,
            u_note_left,
            u_note_right,
            u_note_top,
            u_note_bottom,
            u_particle_left,
            u_particle_right,
            u_particle_top,
            u_particle_bottom,
            u_particle_transparency,
        }
    }
}
