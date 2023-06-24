extern crate gl;
extern crate sdl2;

use crate::midi::midi_to_vertices;

#[derive(Clone)]
pub struct OpenGLContext {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub frame: usize,
    pub data: Vec<u8>,

    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,

    pub vbo: gl::types::GLuint,
    pub vao: gl::types::GLuint,
    pub ibo: gl::types::GLuint,
}

impl OpenGLContext {
    pub fn new(width: usize, height: usize, frame: usize) -> Self {
        let bytes: usize = width*height*4;
        let data: Vec<u8> = vec![0 ; bytes];

        let (vertices, indices) = midi_to_vertices(frame);

        let vbo: gl::types::GLuint = 0;
        let vao: gl::types::GLuint = 0;
        let ibo: gl::types::GLuint = 0;

        OpenGLContext {
            width,
            height,
            bytes,
            frame,
            data,

            vertices,
            indices,

            vbo,
            vao,
            ibo,
        }
            .setup_vbo()
            .setup_vao()
            .setup_context()
            .setup_ibo()
    }

    pub fn setup_vbo(mut self) -> Self {
        unsafe {
            gl::GenBuffers(1, &mut self.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
        self
    }
    
    pub fn setup_vao(mut self) -> Self {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);

            gl::BindVertexArray(self.vao);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                1,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
        }
        self
    }
    
    pub fn setup_context(self) -> Self {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        self
    }

    pub fn setup_ibo(mut self) -> Self {
        unsafe {
            gl::GenBuffers(1, &mut self.ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                self.indices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
        self
    }
}

impl Drop for OpenGLContext{
    fn drop(&mut self) {
        unsafe { 
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}