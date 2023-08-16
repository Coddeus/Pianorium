// use std::sync::{Arc, Mutex};

use egui_sdl2_gl::gl;

use crate::{Notes, Program, Uniform, create_program, Particles, Vao, Vbo, Ibo};


pub struct OpenGLContext {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub cores: usize,

    pub frame: usize,
    pub framerate: f32,
    pub speed: f32,
    pub max_frame: usize,

    pub data: Vec<u8>,

    pub notes: Notes,
    pub particles: Particles,

    pub vbo: Vbo,
    pub vao: Vao,
    pub ibo: Ibo,

    pub program: Program,
    pub u_time: Uniform,
    pub u_resolution: Uniform,
}

// pub struct Shared { // Read-only
// 
// }
// 
// impl std::fmt::Debug for OpenGLContext {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(f, "ogl{}", self.shared.frame)
//     }
// }
// 
// impl Clone for OpenGLContext{
//     fn clone(&self) -> OpenGLContext {
// 
//         OpenGLContext {
//             width: self.width,
//             height: self.height,
//             bytes: self.bytes,
//             cores: self.cores,
//             framerate: self.framerate,
//             speed: self.speed,
//             max_frame: self.max_frame,
//         
//             data: self.data.clone(),
// 
//             shared: self.shared.clone(),
//         }
//             .setup_vbo()
//             .setup_vao()
//             .setup_context()
//             .setup_ibo()
//     

// impl Clone for OpenGLContext{
//     fn clone(&self) -> OpenGLContext {
//         OpenGLContext {
//             width: self.width,
//             height: self.height,
//             bytes: self.bytes,
//             cores: self.cores,
//             
//             frame: self.frame,
//             framerate: self.framerate,
//             speed: self.speed,
//             max_frame: self.max_frame,
//             
//             data: self.data.clone(),
//             
//             notes: self.notes.clone(),
//             particles: self.particles.clone(),
//             
//             vbo: self.vbo,
//             vao: self.vao,
//             ibo: self.ibo,
//             
//             program: self.program.clone(),
//             u_time: self.u_time.clone(),
//             u_resolution: self.u_resolution.clone(),
//         }
//             .setup_vbo()
//             .setup_vao()
//             .setup_context()
//             .setup_ibo()
//     }
// }

impl OpenGLContext {
    pub fn new(width: usize, height: usize, framerate: f32, cores: usize, midi_file: &str) -> Self {
        let bytes: usize = width*height*4;
        let data: Vec<u8> = vec![0 ; bytes];

        let speed: f32 = cores as f32/framerate;
        let frame: usize = 0;
        let (notes, max_frame) = Notes::from_midi(framerate, midi_file).unwrap();

        let particles: Particles = Particles::new();

        let program: Program = create_program().unwrap();
        let u_time: Uniform = Uniform::new(program.id, "u_time").unwrap();
        let u_resolution: Uniform = Uniform::new(program.id, "u_resolution").unwrap();

        unsafe { 
            gl::Uniform1f(u_time.id, 0.0);
            gl::Uniform2f(u_resolution.id, width as f32, height as f32);
        }

        let vbo: Vbo = Vbo::gen(); vbo.set(&notes.vert);
        let vao: Vao = Vao::gen(); vao.set();
        let ibo: Ibo = Ibo::gen(); ibo.set(&notes.ind);
        
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }

        OpenGLContext {
            width,
            height,
            bytes,
            cores,
            
            frame,
            framerate,
            speed,
            max_frame,
            
            data,
            
            notes,
            particles,
            
            vbo,
            vao,
            ibo,
            
            program,
            u_time,
            u_resolution,
        }
    }

    pub fn to_zero(&mut self) {
        let units: f32 = self.speed / self.cores as f32 * self.frame as f32;
        self.frame = 0;
        self.particles = Particles::new();
        self.update(-units);

    }
}

// pub fn fill_handles(width: usize, height: usize, framerate: f32, cores: usize, midi_file: &str) -> Result<Vec<std::thread::JoinHandle<OpenGLContext>>, &'static str> {
//     let mut ogls: Vec<OpenGLContext> = vec![OpenGLContext::new(width, height, framerate, cores, midi_file)];
//     for _u in 1..cores {
//         ogls.push(ogls[ogls.len()-1].clone());
//     }
// 
//     let mut handles: Vec<std::thread::JoinHandle<OpenGLContext>> = vec![];
//     for _u in 0..cores {
//         let ogl = ogls.remove(0);
//         handles.push(std::thread::spawn(move || {ogl}));
//     }
//     Ok(handles)
// }