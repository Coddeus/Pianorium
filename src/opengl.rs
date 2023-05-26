use crate::shaders;

use std::fs::{create_dir, remove_dir_all, File};

extern crate gl;
extern crate num_cpus;
extern crate sdl2;

pub struct OpenGL {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub cores: usize,
    pub frame: usize,
    pub index: File,
    pub data: Vec<Vec<u8>>,
    pub vertices: Vec<f32>,
    pub vao: gl::types::GLuint,
    pub buffers: Vec<gl::types::GLuint>,
    // pub handles: Vec<std::thread::JoinHandle<()>>,
    pub window: sdl2::video::Window,
}

impl OpenGL {
    pub fn new(width: usize, height: usize, window: sdl2::video::Window) -> Self {
        let bytes: usize = width*height*4;
        let cores: usize = num_cpus::get();
        let frame: usize = 0;
        let index: File = File::create("index.txt").unwrap();
        let data: Vec<Vec<u8>> = vec![vec![0 ; bytes] ; cores];
        let vertices: Vec<f32> = vec![
            //  positions  |   colors
            0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
            -0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
            ]; 
        let vao: gl::types::GLuint = 0;
        let buffers: Vec<gl::types::GLuint> = vec![0 ; cores];

        OpenGL {
            width,
            height,
            bytes,
            cores,
            frame,
            index,
            data,
            vertices,
            vao,
            buffers,
            window,
        }
            .setup_fs()
            .create_program()
            .setup_vao()
            .setup_buffers()
            .setup_context()
    }
    
    pub fn setup_fs(self) -> Self {
        match remove_dir_all("temp"){
            _ => {}
        };
        create_dir("temp").unwrap();
        self
    }
    
    pub fn teardown_fs() {
        match remove_dir_all("temp"){
            _ => {}
        };
    }
    
    pub fn create_program(self) -> Self {
        use std::ffi::CString;

        let vert_shader =
            shaders::Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

        let frag_shader =
            shaders::Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();
        
        let shader_program = shaders::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
        shader_program.set_used();
        self
    }
}

impl Drop for OpenGL {
    fn drop(&mut self) {
        unsafe { 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            for vbo in self.buffers.iter_mut() { gl::DeleteBuffers(1, vbo); }
            gl::DeleteVertexArrays(1, &self.vao);
        }

        OpenGL::concat_output();
        OpenGL::teardown_fs();
    }
}