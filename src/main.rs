use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;
use std::process::{Command, Stdio};
use std::ptr::null;

extern crate gl;
extern crate sdl2;

pub mod render_gl;

fn concat_output() {
    Command::new("ffmpeg")
        .env("FFREPORT", "file=ffconcat.log:level=56")
        .arg("-loglevel")
        .arg("0")
        .arg("-f")
        .arg("concat")
        .arg("-i")
        .arg("index.txt")
        .arg("-framerate")
        .arg("60")
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg("output.mp4")
        .output()
        .unwrap();

    println!("\n✨ Fresh video generated! ✨\n");
}

fn main() {
    match remove_dir_all("temp"){
        _ => {}
    };

    create_dir("temp").unwrap();

    let mut index_file = File::create("index.txt").unwrap();



    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // TODO display window ? => if not: good perf [might choose to use only fbo => current state ; but tex not read]
    let window = video_subsystem
        .window("Pianorium", 900, 700)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);



    use std::ffi::CString;

    let vert_shader =
        render_gl::Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

    let frag_shader =
        render_gl::Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();

    let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();



    let vertices: Vec<f32> = vec![
        //  positions  |   colors
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
        0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
    ];


    
    let mut vao: gl::types::GLuint = 0;
    let mut vbo: gl::types::GLuint = 0;
    let mut fbo: gl::types::GLuint = 0;
    let mut texture: gl::types::GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut fbo);
        gl::GenTextures(1, &mut texture);
    }

    unsafe {
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(),                                     // offset of the first component
        );
        gl::EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            1,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );


        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,                                                       // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW,                               // usage
        );


        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::DrawBuffer(gl::COLOR_ATTACHMENT0);


        // TODO https://learnopengl.com/Advanced-OpenGL/Anti-Aliasing
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            texture,
            0,
            gl::RGBA32I as i32,
            900,
            700,
            0,
            gl::RGBA32I,
            gl::UNSIGNED_BYTE,
            null(),
        );
        gl::TexParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);
    }



    unsafe {
        gl::Viewport(0, 0, 900, 700);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }



    let mut pixel_data: Vec<u8> = vec![0; 4 * 900 * 700];
    
    let mut i: u32 = 0;
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }



        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                3,
            );
        }

        shader_program.set_used();
        
        

        window.gl_swap_window();



        // TODO Remove readpixels (and pixel_data) and pass texture data directly to ffmpeg
        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(
                0,
                0,
                900,
                700,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixel_data.as_mut_ptr() as *mut gl::types::GLvoid,
            );
        }



        let name = format!("temp/{:010}.mp4", i);
        let filename = name.as_str();

        let mut ffmpeg = Command::new("ffmpeg")
        .env("FFREPORT", "file=ffreport.log:level=56")
        .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-s")
            .arg(format!("{}x{}", 900, 700))
            .arg("-i")
            .arg("-")
            .arg("-vcodec")
            .arg("libx264")
            .arg("-crf")
            .arg("23")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&pixel_data).unwrap();
        }
        
        writeln!(index_file, "file {}", filename).unwrap();



        println!("Frame {i} generated!");
        i += 1;
    }
    
    

    unsafe {
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteFramebuffers(1, &fbo);
        gl::DeleteTextures(1, &texture);
    }

    concat_output();
}





















// fn main() {
//     match remove_dir_all("temp"){
//         Ok(_existed) => {},
//         Err(_notexisted) => {}
//     };
//     create_dir("temp").unwrap();
//     let mut index_file = File::create("index.txt").unwrap();

//     let sdl = sdl2::init().unwrap();
//     let video_subsystem = sdl.video().unwrap();

//     let gl_attr = video_subsystem.gl_attr();

//     gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
//     gl_attr.set_context_version(3, 3);

//     let window = video_subsystem
//         .window("Game", 900, 700)
//         .opengl()
//         .resizable()
//         .build()
//         .unwrap();

//     let _gl_context = window.gl_create_context().unwrap();
//     let _gl =
//         gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

//     // set up shader program

//     use std::ffi::CString;
//     let vert_shader =
//         render_gl::Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

//     let frag_shader =
//         render_gl::Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();

//     let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

//     // set up vertex buffer object

//     let vertices: Vec<f32> = vec![
//         // positions      // colors
//         0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
//         -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
//         0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
//     ];

//     let mut vbo: gl::types::GLuint = 4;
//     let mut pbo: gl::types::GLuint = 3;
//     let mut fbo: gl::types::GLuint = 2;
//     let mut texture: gl::types::GLuint = 1;
//     unsafe {
//         gl::GenBuffers(1, &mut vbo);
//         gl::GenBuffers(1, &mut pbo);
//         gl::GenFramebuffers(1, &mut fbo);
//         gl::GenTextures(1, &mut texture)
//     }

//     let mut pixel_data: Vec<u8> = vec![0; 900 * 700 * 4];
//     unsafe {
//         gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
//         gl::BufferData(
//             gl::ARRAY_BUFFER, // target
//             (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
//             vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
//             gl::STATIC_DRAW, // usage
//         );
//         gl::BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer

//         gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
//         gl::BufferData(
//             gl::PIXEL_PACK_BUFFER,                               // target
//             900 * 700 * 4 as gl::types::GLsizeiptr,              // size of data in bytes
//             pixel_data.as_mut_ptr() as *const gl::types::GLvoid, // pointer to data
//             gl::STREAM_DRAW,                                     // usage
//         );
//         gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);

//         gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
//         gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
//         gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

//         gl::BindTexture(gl::TEXTURE_2D, texture);
//         gl::TexImage2D(
//             texture,
//             0,
//             gl::RGBA32I as i32,
//             900,
//             700,
//             0,
//             gl::RGBA32I,
//             gl::UNSIGNED_BYTE,
//             null(),
//         );
//         gl::TexParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
//         gl::TexParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
//         gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);
//         gl::BindTexture(gl::TEXTURE_2D, 0);
//     }

//     // set up vertex array object

//     let mut vao: gl::types::GLuint = 0;
//     unsafe {
//         gl::GenVertexArrays(1, &mut vao);
//     }

//     unsafe {
//         gl::BindVertexArray(vao);
//         gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

//         gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
//         gl::VertexAttribPointer(
//             0,         // index of the generic vertex attribute ("layout (location = 0)")
//             3,         // the number of components per generic vertex attribute
//             gl::FLOAT, // data type
//             gl::FALSE, // normalized (int-to-float conversion)
//             (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
//             std::ptr::null(),                                     // offset of the first component
//         );
//         gl::EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
//         gl::VertexAttribPointer(
//             1,         // index of the generic vertex attribute ("layout (location = 0)")
//             3,         // the number of components per generic vertex attribute
//             gl::FLOAT, // data type
//             gl::FALSE, // normalized (int-to-float conversion)
//             (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
//             (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
//         );

//         gl::BindBuffer(gl::ARRAY_BUFFER, 0);
//         gl::BindVertexArray(0);
//     }

//     // set up shared state for window

//     unsafe {
//         gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
//         gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
//         gl::Viewport(0, 0, 900, 700);
//         gl::ClearColor(0.3, 0.3, 0.5, 0.01);
//     }
//     let mut pixel_data: Vec<u8> = vec![0;4*900*700];

//     // main loop
//     let mut i = 0;
//     let mut event_pump = sdl.event_pump().unwrap();
//     'main: loop {
//         for event in event_pump.poll_iter() {
//             match event {
//                 sdl2::event::Event::Quit { .. } => break 'main,
//                 _ => {}
//             }
//         }

//         unsafe {
//             gl::Clear(gl::COLOR_BUFFER_BIT);
//         }

//         // draw triangle

//         shader_program.set_used();
//         unsafe {
//             gl::BindVertexArray(vao);
//             gl::DrawArrays(
//                 gl::TRIANGLES, // mode
//                 0,             // starting index in the enabled arrays
//                 3,             // number of indices to be rendered
//             );
//         }

//         window.gl_swap_window();
//         unsafe {
//             gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
//             gl::ReadPixels(0, 0, 900, 700, gl::RGBA, gl::UNSIGNED_BYTE, pixel_data.as_mut_ptr() as *mut gl::types::GLvoid);
//         }

//         let name = format!("temp/{:010}.mp4", i);
//         let filename = name.as_str();
//         let mut ffmpeg = Command::new("ffmpeg")
//             .env("FFREPORT", "file=ffreport.log:level=56")
//             .arg("-loglevel")
//             .arg("0")
//             .arg("-f")
//             .arg("rawvideo")
//             .arg("-pix_fmt")
//             .arg("rgba")
//             .arg("-s")
//             .arg(format!("{}x{}", 900, 700))
//             .arg("-i")
//             .arg("-")
//             .arg("-vcodec")
//             .arg("libx264")
//             .arg("-crf")
//             .arg("23")
//             .arg(filename)
//             .stdin(Stdio::piped())
//             .spawn()
//             .unwrap();

//         if let Some(ref mut stdin) = ffmpeg.stdin {
//             stdin
//                 .write_all(&pixel_data)
//                 .unwrap();
//         }
//         writeln!(index_file, "file {}", filename).unwrap();

//         println!("{i}");
//         i += 1;
//     }

//     unsafe {
//         gl::DeleteFramebuffers(1, &fbo);
//         gl::DeleteBuffers(1, &pbo);
//         gl::DeleteTextures(1, &texture);
//     }
//     concat_output();
// }