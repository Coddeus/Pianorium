pub mod opengl;
pub mod shaders;
pub mod objects;
pub mod ffmpeg;

fn main() {
    let sdl: sdl2::Sdl = sdl2::init().unwrap();
    let video_subsystem: sdl2::VideoSubsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window: sdl2::video::Window = video_subsystem
        .window("Pianorium", 900, 700)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);


    let mut ogl: opengl::OpenGL = opengl::OpenGL::new(900, 700, window);


    let mut event_pump = sdl.event_pump().unwrap();
    'main: while ogl.frame<250 {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        let modulo = ogl.frame % ogl.cores;
        ogl.draw(modulo);
        ogl.export(modulo);
        
        println!("Frame {} generated!", ogl.frame);
        ogl.frame += 1;
    }
}