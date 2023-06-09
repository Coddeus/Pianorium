mod objects;
mod shaders;

extern crate gl;
extern crate sdl2;

#[derive(Clone)]
pub struct OpenGLContext {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub frame: usize,
    pub data: Vec<u8>,
    pub vertices: Vec<f32>,
    pub vao: gl::types::GLuint,
    pub buffer: gl::types::GLuint,
}

impl OpenGLContext {
    pub fn new(width: usize, height: usize, frame: usize) -> Self {
        let bytes: usize = width*height*4;
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
            width,
            height,
            bytes,
            frame,
            data,
            vertices,
            vao,
            buffer,
        }
            .create_program()
            .setup_buffer()
            .setup_vao()
            .setup_context()
    }
    
    fn create_program(self) -> Self {
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

impl Drop for OpenGLContext{
    fn drop(&mut self) {
        unsafe { 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            gl::DeleteBuffers(1, &self.buffer);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}