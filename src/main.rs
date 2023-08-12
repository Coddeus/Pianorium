extern crate gl;
extern crate sdl2;
// extern crate egui;
extern crate num_cpus;

pub mod fs;
pub mod opengl;
pub mod midi;
pub mod render;
pub mod ui;
pub mod window;

use ui::cli::Parameters;
use opengl::shaders::create_program;
use opengl::context::{OpenGLContext, fill_handles};
use opengl::uniforms::Uniform;

use std::io::Write;
// use std::time::Instant;

fn main() {
    // INPUT ARGS
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let params: Parameters = Parameters::build(&args).unwrap();

    // FS
    fs::setup().unwrap();
    let mut index_file = std::fs::File::create("index.txt").unwrap();

    // WINDOW
    let winsdl: window::Winsdl = window::Winsdl::new(params.width, params.height, params.samples).unwrap();

    // SHADER
    let shader: gl::types::GLuint = create_program();

    // EGUI
    // let shader_ver = egui_sdl2_gl::ShaderVersion::Default;
    // let (mut painter, mut egui_state) =
    //     egui_sdl2_gl::with_sdl2(&window, shader_ver, egui_sdl2_gl::DpiScaling::Custom(2.0));
    // let mut egui_ctx = egui::CtxRef::default();

    // UNIFORMS
    let u_time: Uniform = Uniform::new(shader, "u_time").unwrap();
    unsafe { gl::Uniform1f(u_time.id, 0.0); }

    let u_resolution: Uniform = Uniform::new(shader, "u_resolution").unwrap();
    unsafe { gl::Uniform2f(u_resolution.id, params.width as f32, params.height as f32); }
    
    let mut handles: Vec<std::thread::JoinHandle<OpenGLContext>> = fill_handles(params.width, params.height, params.framerate, params.cores, params.midi_file).unwrap();


    println!("{:?}", handles);
    // let start_time = Instant::now();
    // let mut slider = 0.0;
    let mut event_pump: sdl2::EventPump = winsdl.sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {  } // egui_state.process_input(&window, event, &mut painter);
            }
        }

        for _u in 0..params.cores {
            let mut ogl = handles.remove(0).join().unwrap();
            if ogl.frame > ogl.shared.max_frame { break 'main; }                                   // Stop when it's finished playing
            unsafe { gl::Uniform1f(u_time.id, ogl.frame as f32/params.framerate); }            
            ogl.draw();
            ogl.read();
            winsdl.window.gl_swap_window();
            let name: String = format!("temp/{:010}.mp4", ogl.frame);
            let filename: &str = name.as_str();
            writeln!(index_file, "file {}", filename).unwrap();
            println!("Frame {} generated!", ogl.frame);
            handles.push(std::thread::spawn(move ||{
                ogl.export();
                ogl.frame += params.cores;
                ogl
            }));
        }
    }
    
    render::concat_output(params.output_file); // ≃1/4 of runtime
    if params.clear_dir { fs::teardown().unwrap(); }
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