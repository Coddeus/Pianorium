mod objects;
mod shaders;

use std::fs::{create_dir, remove_dir_all};

extern crate gl;
extern crate num_cpus;
extern crate sdl2;

#[derive(Clone)]
pub struct OpenGLContext {
    pub data: Vec<u8>,
    pub vertices: Vec<f32>,
    pub vao: gl::types::GLuint,
    pub buffer: gl::types::GLuint,
}

impl OpenGLContext {
    pub fn new(bytes: usize) -> Self {
        let data: Vec<u8> = vec![0 ; bytes];
        let vertices: Vec<f32> = vec![
            //  positions  |   colors
            0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
            -0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
        ]; 
        let vao: gl::types::GLuint = 0;
        let buffer: gl::types::GLuint = 0;

        OpenGLContext {
            data,
            vertices,
            vao,
            buffer
        }
        .setup()
    }

    fn setup(self) -> Self{
        self
            .setup_buffer()
            .setup_vao()
            .setup_context()
    }
}

impl Drop for OpenGLContext {
    fn drop(&mut self) {
        unsafe { 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            gl::DeleteBuffers(1, &self.buffer);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}


pub struct OpenGL {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub cores: usize,
    pub frame: usize,
    pub window: sdl2::video::Window,
    pub contexts: Vec<OpenGLContext>,
}

impl OpenGL {
    pub fn new(width: usize, height: usize, window: sdl2::video::Window) -> Self {
        let bytes: usize = width*height*4;
        let cores: usize = num_cpus::get();
        let frame: usize = 0;
        let contexts: Vec<OpenGLContext> = vec![OpenGLContext::new(bytes) ; cores];

        OpenGL {
            width,
            height,
            bytes,
            cores,
            frame,
            window,
            contexts,
        }
            .setup_fs()
            .create_program()
    }
    
    pub fn setup_fs(self) -> Self {
        match remove_dir_all("temp"){
            _ => {}
        };
        create_dir("temp").unwrap();
        self
    }

    fn teardown_fs(&self) -> &Self{
        match remove_dir_all("temp"){
            _ => {}
        };
        // index.txt
        // ffreport.log
        // ffconcat.log
        self
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

    pub fn render_frame(mut self) -> Self {
        let num = self.frame;

        self.contexts[num%self.cores]
            .draw()
            .read();
        self.window.gl_swap_window();
        self.contexts[num%self.cores]
            .export(num);
        self
    }
}

impl Drop for OpenGL{
    fn drop(&mut self) {
        self
        .concat_output()
        .teardown_fs();
    }
}