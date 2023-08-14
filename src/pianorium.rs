use std::{/*thread::JoinHandle,*/ env::args};

use crate::{Winsdl, OpenGLContext, /*fill_handles, */Parameters, fs, Gui};


pub struct Pianorium { // Options for everything & mode chosen
    pub params: Parameters,
    pub winsdl: Winsdl,
    pub ogl: OpenGLContext,
    pub gui: Gui,
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
        // let handles: Vec<JoinHandle<OpenGLContext>> = fill_handles(params.width, params.height, params.framerate, params.cores, &params.midi_file).unwrap();
        let ogl = OpenGLContext::new(params.width, params.height, params.framerate, params.cores, &params.midi_file);

        let gui = Gui::new(&winsdl.window).unwrap();

        Ok(Pianorium {
            params,
            winsdl,
            ogl,
            gui,
        })
    }
}