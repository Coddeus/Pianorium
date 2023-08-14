use egui_sdl2_gl::gl;

use crate::OpenGLContext;


impl OpenGLContext {
    pub fn draw(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            self.draw_notes();
            self.draw_particles();
        }
    }

    pub fn draw_notes(&mut self) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.notes.vert.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.notes.vert.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.notes.ind.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                self.notes.ind.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::DrawElements(gl::TRIANGLES, self.notes.ind.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
        }
    }
    
    pub fn draw_particles(&mut self) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.particles.particle_vert.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.particles.particle_vert.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.particles.particle_ind.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                self.particles.particle_ind.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::DrawElements(gl::TRIANGLES, self.particles.particle_ind.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
        }
    }

    pub fn update(&mut self, diff: f32) {
        for y in self.notes.vert
            .iter_mut()
            .skip(1)
            .step_by(3) 
        {
            *y-=diff;
        }

        self.particles.update(diff, &self.notes.vert);
    }
}