use egui_sdl2_gl::gl;

use crate::OpenGLContext;


impl OpenGLContext {
    pub fn draw(&mut self, rgb: [f32 ; 3], since_start: f32) {
        unsafe {
            self.vbo.set(&self.notes.vert);
            self.vao.set();
            self.ibo.set(&self.notes.ind);
            self.program.set_used();
            gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
            gl::Uniform1f(self.u_time.id, since_start as f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            self.draw_notes();
            self.draw_particles();
        }
    }

    pub fn draw_notes(&mut self) {
        unsafe {
            self.vbo.set(&self.notes.vert);
            self.ibo.set(&self.notes.ind);
            gl::DrawElements(gl::TRIANGLES, self.notes.ind.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
        }
    }
    
    pub fn draw_particles(&mut self) {
        unsafe {
            self.vbo.set(&self.particles.vert);
            self.ibo.set(&self.particles.ind);
            gl::DrawElements(gl::TRIANGLES, self.particles.ind.len() as i32, gl::UNSIGNED_INT, 0 as *const _);
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