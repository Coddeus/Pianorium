impl crate::OpenGLContext {
    pub fn draw(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );

            for y in self.vertices
                .iter_mut()
                .skip(1)
                .step_by(5) 
            {
                *y-=0.02;
            }

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}