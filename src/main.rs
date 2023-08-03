extern crate gl;
extern crate sdl2;
extern crate num_cpus;

pub mod cli;
pub mod drawing;
pub mod ffmpeg;
pub mod fs;
pub mod midi;
pub mod opengl;
pub mod shaders;

use std::fs::File;
use std::io::Write;
use opengl::OpenGLContext;
                                                       
fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let params = cli::Parameters::build(&args).unwrap();
    
    let width: usize = params.width;
    let height: usize = params.height;
    let framerate: f32 = params.framerate;
    let samples: u8 = params.samples;
    let midi_file: String = params.midi_file;
    let output_file: String = params.output_file;
    let clear_dir: bool = params.clear_dir;

    let threads: usize = num_cpus::get_physical();
    println!("You have {} CPU cores", threads);


    fs::setup();
    let mut index_file = File::create("index.txt").unwrap();

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

    let shader: gl::types::GLuint = shaders::create_program();

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
    
    let mut ogl1: OpenGLContext = opengl::OpenGLContext::new(width, height, framerate, midi_file);
    let mut ogl2: OpenGLContext = ogl1.clone();
    let mut ogl3: OpenGLContext = ogl2.clone();
    let mut ogl4: OpenGLContext = ogl3.clone();
    let mut ogl5: OpenGLContext = ogl4.clone();
    let mut ogl6: OpenGLContext = ogl5.clone();

    let mut handle1: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl1});
    let mut handle2: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl2});
    let mut handle3: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl3});
    let mut handle4: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl4});
    let mut handle5: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl5});
    let mut handle6: std::thread::JoinHandle<OpenGLContext> = std::thread::spawn(move || {ogl6});

    let mut i: usize = 0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        
        ogl1 = handle1.join().unwrap();
        if i > ogl1.max_frame { break 'main; }                                   // Stop when it's finished playing
        unsafe { gl::Uniform1f(location_utime, ogl1.frame as f32/60.); }
        ogl1.draw();
        ogl1.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl1.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i);
        handle1 = std::thread::spawn(move ||{
            ogl1.export();
            ogl1.frame += 6;
            ogl1
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
            ogl2.frame += 6;
            ogl2
        });
        
        ogl3 = handle3.join().unwrap();
        unsafe { gl::Uniform1f(location_utime, ogl3.frame as f32/60.); }
        ogl3.draw();
        ogl3.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl3.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i+2);
        handle3 = std::thread::spawn(move ||{
            ogl3.export();
            ogl3.frame += 6;
            ogl3
        });
        
        ogl4 = handle4.join().unwrap();
        unsafe { gl::Uniform1f(location_utime, ogl4.frame as f32/60.); }
        ogl4.draw();
        ogl4.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl4.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i+3);
        handle4 = std::thread::spawn(move ||{
            ogl4.export();
            ogl4.frame += 6;
            ogl4
        });
        
        ogl5 = handle5.join().unwrap();
        unsafe { gl::Uniform1f(location_utime, ogl5.frame as f32/60.); }
        ogl5.draw();
        ogl5.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl5.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i+4);
        handle5 = std::thread::spawn(move ||{
            ogl5.export();
            ogl5.frame += 6;
            ogl5
        });
        
        ogl6 = handle6.join().unwrap();
        unsafe { gl::Uniform1f(location_utime, ogl6.frame as f32/60.); }
        ogl6.draw();
        ogl6.read();
        window.gl_swap_window();
        let name: String = format!("temp/{:010}.mp4", ogl6.frame);
        let filename: &str = name.as_str();
        writeln!(index_file, "file {}", filename).unwrap();
        println!("Frame {} generated!", i+5);
        handle6 = std::thread::spawn(move ||{
            ogl6.export();
            ogl6.frame += 6;
            ogl6
        });
        
        i+=6;
    }
    
    ffmpeg::concat_output(output_file); // ≃1/4 of runtime
    if clear_dir { fs::teardown(); }
}