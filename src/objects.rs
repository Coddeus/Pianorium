use crate::opengl::OpenGL;

impl OpenGL {
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

    pub fn setup_buffers(mut self) -> Self {
        unsafe {
            for vbo in self.buffers.iter_mut() {
                gl::GenBuffers(1, vbo);
                gl::BindBuffer(gl::ARRAY_BUFFER, *vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    self.vertices.as_ptr() as *const gl::types::GLvoid,
                    gl::DYNAMIC_DRAW,
                );
            }
        }
        self
    }

    pub fn setup_context(self) -> Self {
        unsafe {
            gl::Viewport(0, 0, 900, 700);
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        self
    }

    pub fn draw(&self, modulo: usize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffers[modulo]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}
