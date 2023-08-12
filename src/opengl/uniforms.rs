use gl::types::GLint;

pub struct Uniform {
    pub id: GLint,
}

impl Uniform {
    pub fn new(shader: u32, name: &str) -> Result<Self, &'static str> {
        let cname: std::ffi::CString = std::ffi::CString::new(name).expect("CString::new failed");
        let location: gl::types::GLint;
        unsafe {
            location = gl::GetUniformLocation(shader, cname.as_ptr());
        }
        Ok(Uniform{ id: location })
    }
}