use std::process::{Command, Stdio};
use std::io::Write;

impl crate::OpenGLContext {
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
    
    pub fn export(&self) {
        let name = format!("temp/{:010}.mp4", self.frame);
        let filename = name.as_str();

        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=ffreport.log:level=56")
            .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("rawvideo")
            .arg("-r")
            .arg("60")
            .arg("-pix_fmt")
            .arg("bgra")
            .arg("-s")
            .arg(format!("{}x{}", self.width, self.height))
            .arg("-i")
            .arg("-")
            .arg("-vcodec")
            .arg("libx264")
            .arg("-crf")
            .arg("23")
            .arg("-vf")
            .arg("vflip")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&self.data).unwrap();
        }
    }
}

pub fn concat_output(output: String) {
    println!("\nConcatenating into one video…\n");

    Command::new("ffmpeg")
        .env("FFREPORT", "file=ffconcat.log:level=56")
        .arg("-loglevel")
        .arg("0")
        .arg("-f")
        .arg("concat")
        .arg("-i")
        .arg("index.txt")
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output.as_str())
        .output()
        .unwrap();

    println!("\n✨ Fresh video generated! ✨\n");
}