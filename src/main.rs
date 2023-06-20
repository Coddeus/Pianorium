use std::fs::{create_dir, remove_dir_all, remove_file, File};
use std::io::Write;

use opengl::OpenGLContext;

extern crate gl;
extern crate sdl2;

pub mod drawing;
pub mod ffmpeg;
pub mod opengl;
pub mod shaders;
                                                                
fn main() {
    setup_fs();
    let mut index_file = File::create("index.txt").unwrap();

    let width: usize = 900;
    let height: usize = 700;
    let samples: u8 = 0;
    let clear_dir: bool = true;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);
    if samples>1 {
        gl_attr.set_multisample_samples(samples);
    }

    // TODO display window ? => if not: good perf [might choose to use only fbo => current state ; but tex not read]
    let window = video_subsystem
        .window("Pianorium", width as u32, height as u32)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    create_program();
    
    if samples>1 {
        unsafe {
            gl::Enable(gl::MULTISAMPLE);
        }
    }
    
    let mut ogl: OpenGLContext = opengl::OpenGLContext::new(width, height, 0);
    let mut ogl2: OpenGLContext = opengl::OpenGLContext::new(width, height, 1);
    let mut handle1: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl});
    let mut handle2: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl2});

    let mut i = 0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: while i < 500 {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        
        ogl = handle1.join().unwrap();
        ogl.draw();
        ogl.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i);
        handle1 = std::thread::spawn(move ||{
            ogl.export();
            ogl.frame += 2;
            ogl
        });
        
        ogl2 = handle2.join().unwrap();
        ogl2.draw();
        ogl2.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl2.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i+1);
        handle2 = std::thread::spawn(move ||{
            ogl2.export();
            ogl2.frame += 2;
            ogl2
        });
        
        i+=2;
    }
    
    ffmpeg::concat_output(); // ≃1/4 of runtime
    if clear_dir {teardown_fs();}
}
    
fn create_program() {
    use std::ffi::CString;

    let vert_shader =
        shaders::Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

    let frag_shader =
        shaders::Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();
    
    let shader_program = shaders::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    shader_program.set_used();
}

fn setup_fs() {
    match remove_dir_all("temp"){
        _ => {}
    };
    create_dir("temp").unwrap();
}

fn teardown_fs() {
    match remove_dir_all("temp"){
        _ => {}
    };
    remove_file("index.txt").unwrap();
    remove_file("ffreport.log").unwrap();
    remove_file("ffconcat.log").unwrap();
}