use std::process::{Command, Stdio};
use std::io::Write;

impl crate::opengl::OpenGLContext {
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
            .arg(format!("{}x{}", 900, 700))
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

pub fn concat_output() {
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
        .arg("output.mp4")
        .output()
        .unwrap();

    println!("\n✨ Fresh video generated! ✨\n");
}