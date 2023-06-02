use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;

extern crate gl;
extern crate sdl2;

pub mod ffmpeg;
pub mod opengl;

fn main() {
    match remove_dir_all("temp"){
        _ => {}
    };

    create_dir("temp").unwrap();

    let mut index_file = File::create("index.txt").unwrap();

    let width: usize = 900;
    let height: usize = 700;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(1);
    gl_attr.set_multisample_samples(10);

    // TODO display window ? => if not: good perf [might choose to use only fbo => current state ; but tex not read]
    let window = video_subsystem
        .window("Pianorium", width as u32, height as u32)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::Enable(gl::MULTISAMPLE);
    }

    let mut ogl: opengl::OpenGLContext = opengl::OpenGLContext::new(width, height);
    let mut event_pump = sdl.event_pump().unwrap();

    'main: while ogl.frame < 200 {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let mut ogl2: opengl::OpenGLContext = ogl.clone();

        ogl2.draw();
        ogl2.read();
        window.gl_swap_window();

        std::thread::spawn(move ||{
            ogl2.export();
        });

        let name: String = format!("temp/{:010}.mp4", ogl.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();

        println!("Frame {} generated!", ogl.frame);
        ogl.frame += 1;
    }

    
    ffmpeg::concat_output(); // ≃1/4 of runtime
    teardown_fs();
}

fn teardown_fs() {
    match remove_dir_all("temp"){
        _ => {}
    };
    // index.txt
    // ffreport.log
    // ffconcat.log

}