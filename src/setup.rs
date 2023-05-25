extern crate gl;
extern crate num_cpus;
extern crate sdl2;

pub struct OpenGL {
    width: usize,
    height: usize,
    bytes: usize,
    cores: usize,
    data: Vec<Vec<u8>>,
    vbos: Vec<gl::types::GLuint>,
    vao: gl::types::GLuint,
    window: sdl2::video::Window,
    index: std::fs::File,
    frame: u32,
}

impl OpenGL {
    fn new(width: usize, height: usize) -> OpenGL {
        let video_subsystem = sdl2::init().unwrap().video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let bytes: usize = width*height*4;
        let cores: usize = num_cpus::get_physical();
        let data: Vec<Vec<u8>> = vec![vec![0 ; bytes] ; cores];
        let vbos: Vec<gl::types::GLuint> = vec![0 ; cores];
        let vao: gl::types::GLuint = 0;
        let window: sdl2::video::Window = video_subsystem
            .window("Pianorium", 900, 700)
            .opengl()
            .build()
            .unwrap();
        let index: std::fs::File = std::fs::File::create("index.txt").unwrap();
        let frame: u32 = 0;

        OpenGL {
            width,
            height,
            bytes,
            cores,
            data,
            vbos,
            vao,
            window,
            index,
            frame,
        }
    }
}

impl Drop for OpenGL {
    fn drop(&mut self) {
        unsafe { 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            for vbo in self.vbos.iter_mut() { gl::DeleteBuffers(1, vbo); }
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}