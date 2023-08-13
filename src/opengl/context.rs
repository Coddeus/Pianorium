extern crate gl;
extern crate sdl2;

use crate::{midi_to_vertices, Program, Uniform, create_program};
use gl::types::GLuint;

pub struct OpenGLContext {
    pub frame: usize,

    pub data: Vec<u8>,

    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,

    pub vbo: GLuint,
    pub vao: GLuint,
    pub ibo: GLuint,

    pub shared: std::sync::Arc<Shared>,
}

pub struct Shared {
    pub width: usize,
    pub height: usize,

    pub bytes: usize,
    pub cores: usize,

    pub speed: f32,
    pub framerate: f32,

    pub max_frame: usize,

    pub program: Program,
    pub u_time: Uniform,
    pub u_resolution: Uniform,
}

impl std::fmt::Debug for OpenGLContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ogl{}", self.frame)
    }
}

impl Clone for OpenGLContext{
    fn clone(&self) -> OpenGLContext{
        let data = self.data.clone();
        let mut vertices = self.vertices.clone();
        let indices = self.indices.clone();

        for y in vertices
            .iter_mut()
            .skip(1)
            .step_by(3) 
        {
            *y-=self.shared.speed/self.shared.cores as f32;
        }

        OpenGLContext {
            data,

            frame: self.frame+1,

            vertices,
            indices,

            vbo: self.vbo,
            vao: self.vao,
            ibo: self.ibo,

            shared: self.shared.clone(),
        }
            .setup_vbo()
            .setup_vao()
            .setup_context()
            .setup_ibo()
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

impl OpenGLContext {
    pub fn new(width: usize, height: usize, framerate: f32, cores: usize, midi_file: &str) -> Self {
        let bytes: usize = width*height*4;
        let data: Vec<u8> = vec![0 ; bytes];

        let speed: f32 = cores as f32/framerate;
        let frame: usize = 0;
        let (vertices, indices, max_frame) = midi_to_vertices(framerate, midi_file).unwrap();

        let vbo: GLuint = 0;
        let vao: GLuint = 0;
        let ibo: GLuint = 0;
        
        let program: Program = create_program().unwrap();
        
        let u_time: Uniform = Uniform::new(program.id, "u_time").unwrap();
        let u_resolution: Uniform = Uniform::new(program.id, "u_resolution").unwrap();
        unsafe { gl::Uniform1f(u_time.id, 0.0); }
        unsafe { gl::Uniform2f(u_resolution.id, width as f32, height as f32); }
        

        OpenGLContext {
            data,

            frame,

            vertices,
            indices,

            vbo,
            vao,
            ibo,

            shared: std::sync::Arc::new(Shared {
                width,
                height,
                bytes,
                speed,
                framerate,
                max_frame,
                cores,

                program,
                u_time,
                u_resolution,
            }),
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
            gl::Viewport(0, 0, self.shared.width as i32, self.shared.height as i32);
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

    pub fn to_zero(&mut self) {
        let units: f32 = self.shared.speed / self.shared.cores as f32 * self.frame as f32;
        self.frame = 0;
        self.update(-units);
    }
}

pub fn fill_handles(width: usize, height: usize, framerate: f32, cores: usize, midi_file: &str) -> Result<Vec<std::thread::JoinHandle<OpenGLContext>>, &'static str> {
    let mut ogls: Vec<OpenGLContext> = vec![OpenGLContext::new(width, height, framerate, cores, midi_file)];
    for _u in 1..cores {
        ogls.push(ogls[ogls.len()-1].clone());
    }

    let mut handles: Vec<std::thread::JoinHandle<OpenGLContext>> = vec![];
    for _u in 0..cores {
        let ogl = ogls.remove(0);
        handles.push(std::thread::spawn(move || {ogl}));
    }
    Ok(handles)
}