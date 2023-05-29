use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;

extern crate gl;
extern crate sdl2;
extern crate num_cpus;

pub mod ffmpeg;
pub mod opengl;

fn main() {
    match remove_dir_all("temp"){
        _ => {}
    };

    create_dir("temp").unwrap();

    let mut index_file = File::create("index.txt").unwrap();



    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // TODO display window ? => if not: good perf [might choose to use only fbo => current state ; but tex not read]
    let window = video_subsystem
        .window("Pianorium", 900, 700)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut ogl = opengl::OpenGL::new(900, 700, window);
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        ogl = ogl.render_frame();

        let name = format!("temp/{:010}.mp4", ogl.frame);
        let filename = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();

        println!("Frame {} generated!", ogl.frame);
        ogl.frame += 1;
    }
}