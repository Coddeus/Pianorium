pub struct Winsdl {
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub gl_context: sdl2::video::GLContext,
    pub gl: (),
}

impl Winsdl {
    pub fn new(width: usize, height: usize, samples: u8) -> Result<Self, &'static str> {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        if samples>1 {
            gl_attr.set_multisample_samples(samples);
        }

        let window = video_subsystem
            .window("Pianorium", width as u32, height as u32)
            .opengl()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        let gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        window
            .subsystem()
            .gl_set_swap_interval(sdl2::video::SwapInterval::Immediate)
            .unwrap();

        if samples>1 {
            unsafe {
                gl::Enable(gl::MULTISAMPLE);
            }
        }

        Ok(Winsdl{ 
            sdl, 
            window, 
            gl_context,
            gl,
        })
    }
}