extern crate gl;
extern crate sdl2;

#[derive(Clone)]
pub struct OpenGLContext {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub frame: usize,
    pub start: std::time::SystemTime,
    pub data: Vec<u8>,
    pub vertices: Vec<f32>,
    pub vao: gl::types::GLuint,
    pub buffer: gl::types::GLuint,
}

impl OpenGLContext {
    pub fn new(width: usize, height: usize, frame: usize) -> Self {
        let bytes: usize = width*height*4;
        let start: std::time::SystemTime = std::time::SystemTime::now();
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
            start,
            data,
            vertices,
            vao,
            buffer,
        }
            .setup_buffer()
            .setup_vao()
            .setup_context()
    }
    
    pub fn setup_vao(mut self) -> Self {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);

            gl::BindVertexArray(self.vao);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
        }
        self
    }

    pub fn setup_buffer(mut self) -> Self {
        unsafe {
            gl::GenBuffers(1, &mut self.buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
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