use std::ffi::CString;

use egui_sdl2_gl::gl::{self, types::GLint};


pub struct Uniform {
    pub id: GLint,
}

impl Uniform {
    pub fn new(shader: u32, name: &str) -> Result<Self, &'static str> {
        let cname: CString = CString::new(name).expect("CString::new failed");
        let location: GLint;
        unsafe {
            location = gl::GetUniformLocation(shader, cname.as_ptr());
        }
        Ok(Uniform{ id: location })
    }
}