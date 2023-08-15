use std::ptr::null;

use egui_sdl2_gl::gl::{self,types::GLuint};

pub struct Vao {
    pub id: GLuint,
}

impl Drop for Vao {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Vao {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe { gl::GenVertexArrays(1, &mut id); }
        Vao { id }
    }

    pub fn set(&self) {
        self.bind();
        self.setup();
    }
    
    fn bind(&self) {
        unsafe { gl::BindVertexArray(self.id); }
    }

    fn setup(&self) {
        unsafe {
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
    }

    fn unbind(&self) {
        unsafe { gl::BindVertexArray(0); }
    }

    fn delete(&self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id); }
    }
}

pub struct Vbo {
    pub id: GLuint,
}

impl Drop for Vbo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Vbo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe { gl::GenBuffers(1, &mut id); }
        Vbo { id }
    }

    pub fn set(&self, data: &Vec<f32>) {
        self.bind();
        self.data(data);
    }
    
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.id); }
    }

    fn data(&self, vertices: &Vec<f32>) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn unbind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0); }
    }

    fn delete(&self) {
        unsafe { gl::DeleteBuffers(1, &self.id); }
    }
}

pub struct Ibo {
    pub id: GLuint,
}

impl Drop for Ibo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Ibo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe { gl::GenBuffers(1, &mut id); }
        Ibo { id }
    }

    pub fn set(&self, data: &Vec<u32>) {
        self.bind();
        self.data(data);
    }

    fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id); }
    }

    fn data(&self, indices: &Vec<u32>) {
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn unbind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); }
    }

    fn delete(&self) {
        unsafe { gl::DeleteBuffers(1, &self.id); }
    }
}

pub struct Fbo {
    pub id: GLuint,
}

impl Drop for Fbo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Fbo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe { gl::GenFramebuffers(1, &mut id); }
        Fbo { id }
    }

    pub fn set(&self, texture: GLuint) {
        self.bind();
        self.tex(texture);
    }

    fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id); }
        #[cfg(debug_assertions)]
        self.check()
    }

    fn check(&self) {
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE { 
            println!("🛑 Framebuffer wasn't successfully bound. Error {:#?}", status);
        }
    }

    fn tex(&self, texture: GLuint) {
        unsafe { gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0); }
    }

    fn unbind(&self) { // Back to default window-projected framebuffer
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    fn delete(&self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id); }
    }
}

pub struct Texture {
    pub id: GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Texture {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe { gl::GenTextures(1, &mut id); }
        Texture { id }
    }

    pub fn set(&self, width: i32, height: i32) {
        self.bind();
        self.setup(width, height);
    }

    fn setup(&self, width: i32, height: i32) {
        unsafe { 
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
    }

    fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
    }

    fn unbind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0); }
    }

    fn delete(&self) {
        unsafe { gl::DeleteBuffers(1, &self.id); }
    }
}