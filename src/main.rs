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

    let width: usize = 1920/2;
    let height: usize = 1080/2;
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

    let shader: gl::types::GLuint = create_program();

    let cname_utime: std::ffi::CString = std::ffi::CString::new("u_time").expect("CString::new failed");
    let location_utime: gl::types::GLint;
    let cname_uresolution: std::ffi::CString = std::ffi::CString::new("u_resolution").expect("CString::new failed");
    let location_uresolution: gl::types::GLint;
    unsafe {
        location_utime = gl::GetUniformLocation(shader, cname_utime.as_ptr());
        gl::Uniform1f(location_utime, 0.0);
        location_uresolution = gl::GetUniformLocation(shader, cname_uresolution.as_ptr());
        gl::Uniform2f(location_uresolution, width as f32, height as f32);
    }
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
    'main: while i < 250 {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        
        ogl = handle1.join().unwrap();
        unsafe { gl::Uniform1f(location_utime, ogl.frame as f32/60.); }
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
        unsafe { gl::Uniform1f(location_utime, ogl2.frame as f32/60.); }
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
    
fn create_program() -> gl::types::GLuint {
    use std::ffi::CString;

    let vert_shader =
        shaders::Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

    let frag_shader =
        shaders::Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();
    
    let shader_program = shaders::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    shader_program.set_used();

    shader_program.id()
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