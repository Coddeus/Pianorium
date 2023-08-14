use std::{process::{Command, Stdio}, io::Write};

use egui_sdl2_gl::gl;

use crate::OpenGLContext;


impl OpenGLContext {
    pub fn read(&mut self) {
        unsafe {
            gl::ReadPixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                self.data.as_mut_ptr() as *mut gl::types::GLvoid,
            );
        }
    }
    
    pub fn export_mp4(&self) {
        let name = format!("temp/{:010}.mp4", self.frame);
        let filename = name.as_str();

        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=ff_export_mp4.log:level=56")
            .arg("-loglevel").arg("0")
            .arg("-f").arg("rawvideo")
            .arg("-r").arg(self.framerate.to_string())
            .arg("-pix_fmt").arg("bgra")
            .arg("-s").arg(format!("{}x{}", self.width, self.height))
            .arg("-i").arg("-")
            .arg("-vcodec").arg("libx264")
            .arg("-crf").arg("23")
            .arg("-vf").arg("vflip")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        
        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&self.data).unwrap();
        }
    }
    
    pub fn export_png(&self, filename: &str) {
        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=ff_export_png.log:level=56")
            .arg("-loglevel").arg("0")
            .arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("bgra")
            .arg("-s").arg(format!("{}x{}", self.width, self.height))
            .arg("-i").arg("-")
            .arg("-frames:v").arg("1")
            .arg("-vf").arg("vflip")
            .arg("-y")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg process.");

        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&self.data).unwrap();
        }
    }
}

pub fn concat_mp4(output: &str) {
    println!("Concatenating into one video…");

    Command::new("ffmpeg")
        .env("FFREPORT", "file=ff_concat_mp4.log:level=56")
        .arg("-loglevel")
        .arg("0")
        .arg("-f")
        .arg("concat")
        .arg("-i")
        .arg("index.txt")
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output)
        .output()
        .unwrap();

    println!("✨ Fresh video generated! ✨");
}