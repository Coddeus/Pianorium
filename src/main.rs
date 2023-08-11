extern crate gl;
extern crate sdl2;
// extern crate egui;
extern crate num_cpus;

pub mod fs;
pub mod opengl;
pub mod midi;
pub mod render;
pub mod ui;

use crate::opengl::shaders::create_program;
use crate::ui::cli::Parameters;
use std::fs::File;
use std::io::Write;
// use std::time::Instant;
use opengl::context::OpenGLContext;
                                                       
fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let params = Parameters::build(&args).unwrap();
    
    let width: usize = params.width;
    let height: usize = params.height;
    let framerate: f32 = params.framerate;
    let samples: u8 = params.samples;
    let midi_file: String = params.midi_file;
    let output_file: String = params.output_file;
    let clear_dir: bool = params.clear_dir;
    let cores: usize = num_cpus::get();

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

    let window = video_subsystem
        .window("Pianorium", width as u32, height as u32)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    window
        .subsystem()
        .gl_set_swap_interval(sdl2::video::SwapInterval::VSync)
        .unwrap();

    let shader: gl::types::GLuint = create_program();
    
    // let shader_ver = egui_sdl2_gl::ShaderVersion::Default;
    // let (mut painter, mut egui_state) =
    //     egui_sdl2_gl::with_sdl2(&window, shader_ver, egui_sdl2_gl::DpiScaling::Custom(2.0));
    // let mut egui_ctx = egui::CtxRef::default();

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

    let mut ogls: Vec<OpenGLContext> = vec![OpenGLContext::new(width, height, framerate, cores, midi_file)];
    for _u in 1..cores {
        ogls.push(ogls[ogls.len()-1].clone());
    }

    let mut handles: Vec<std::thread::JoinHandle<OpenGLContext>> = vec![];
    for _u in 0..cores {
        let ogl = ogls.remove(0);
        handles.push(std::thread::spawn(move || {ogl}))
    }

    // let start_time = Instant::now();
    // let mut slider = 0.0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {  } // egui_state.process_input(&window, event, &mut painter);
            }
        }

        for _u in 0..cores {
            let mut ogl = handles.remove(0).join().unwrap();
            if ogl.frame > ogl.max_frame { break 'main; }                                   // Stop when it's finished playing
            unsafe { gl::Uniform1f(location_utime, ogl.frame as f32/framerate); }            
            ogl.draw();
            ogl.read();
            window.gl_swap_window();
            let name: String = format!("temp/{:010}.mp4", ogl.frame);
            let filename: &str = name.as_str();
            writeln!(index_file, "file {}", filename).unwrap();
            println!("Frame {} generated!", ogl.frame);
            handles.push(std::thread::spawn(move ||{
                ogl.export();
                ogl.frame += cores;
                ogl
            }))
        }
    }
    
    render::concat_output(output_file); // ≃1/4 of runtime
    if clear_dir { fs::teardown(); }
}

// fn draw_gui() { // Struct with Impl
// 
//     egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
//     egui_ctx.begin_frame(egui_state.input.take());
//     egui::CentralPanel::default().show(&egui_ctx, |ui| {
//         ui.label(" ");
//         ui.add(egui::Slider::new(&mut slider, 0.0..=50.0).text("Slider"));
//         ui.label(" ");
//     });
//     let (egui_output, paint_cmds) = egui_ctx.end_frame();
//     egui_state.process_output(&window, &egui_output);
//     let paint_jobs = egui_ctx.tessellate(paint_cmds);
//     painter.paint_jobs(None, paint_jobs, &egui_ctx.font_image());
// }