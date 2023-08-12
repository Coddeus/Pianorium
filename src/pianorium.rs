use crate::{Winsdl, OpenGLContext, fill_handles, Parameters, fs};
use std::{thread::JoinHandle, env::args};

pub struct Pianorium {
    pub params: Parameters,
    pub winsdl: Winsdl,
    pub handles: Vec<JoinHandle<OpenGLContext>>,
}

impl Drop for Pianorium {
    fn drop(&mut self) {
        if self.params.clear_dir { fs::teardown().unwrap(); }
    }
}

impl Pianorium {
    pub fn new() -> Result<Self, &'static str> {
        // INPUT ARGS
        let mut args: Vec<String> = args().collect();
        args.remove(0);
        let params: Parameters = Parameters::build(&args).unwrap();
    
        // FS
        fs::setup().unwrap();

        // WINDOW
        let winsdl: Winsdl = Winsdl::new(params.width, params.height, params.samples).unwrap();
        
        // HANDLES FOR OPENGL
        let handles: Vec<JoinHandle<OpenGLContext>> = fill_handles(params.width, params.height, params.framerate, params.cores, &params.midi_file).unwrap();

        // EGUI
        // let shader_ver = egui_sdl2_gl::ShaderVersion::Default;
        // let (mut painter, mut egui_state) =
        //     egui_sdl2_gl::with_sdl2(&window, shader_ver, egui_sdl2_gl::DpiScaling::Custom(2.0));
        // let mut egui_ctx = egui::CtxRef::default();

        // UNIFORMS

        Ok(Pianorium {
            params,
            winsdl,
            handles,
        })
    }
}