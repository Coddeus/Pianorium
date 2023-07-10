pub const SPEED: f32 = 0.02; // Doesn't change the pace of the song, changes the visual speed of the rectangles (lower value => taller notes falling faster)

impl crate::OpenGLContext {
    pub fn draw(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const _);

            for y in self.vertices
                .iter_mut()
                .skip(1)
                .step_by(3) 
            {
                *y-=2.*SPEED;
            }
        }
    }
}