use std::{fs::File, io::Write, time::Instant, ffi::c_void, slice::from_raw_parts};

use egui_sdl2_gl::{sdl2::{video::SwapInterval, event::Event}, gl};

use crate::{concat_mp4, Pianorium, Fbo, Texture, Pbo};


impl Pianorium {
    pub fn full_mp4(&mut self) -> Result<(), String> {
        self.ogl.to_zero();
        self.winsdl.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();

        let mut index = File::create(self.params.index_file.clone()).unwrap();
        println!("Rendering frames…");

        self.ogl.vbo.set(&self.ogl.notes.vert);
        self.ogl.vao.set();
        self.ogl.ibo.set(&self.ogl.notes.ind);
        self.ogl.program.set_used();

        let tex = Texture::gen();
        tex.set(self.ogl.width as i32, self.ogl.height as i32);
        let fbo = Fbo::gen();
        fbo.set(tex.id);

        let pbo = Pbo::gen();
        pbo.set(self.ogl.bytes);

       unsafe { gl::ReadBuffer(gl::COLOR_ATTACHMENT0); } 
        
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        }
    
        'record: loop {
            for event in self.winsdl.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'record,
                    _ => {  } // egui_state.process_input(&window, event, &mut painter);
                }
            }
    
            // for _u in 0..self.params.cores {
                // let mut ogl = self.handles.remove(0).join().unwrap();
                if self.ogl.frame > self.ogl.max_frame { break 'record; }             // Stop when it's finished playing
                unsafe { gl::Uniform1f(self.ogl.u_time.id, self.ogl.frame as f32/self.params.framerate); }
                
                self.ogl.update(1.0/self.ogl.framerate );                
                self.ogl.draw();

                let time = Instant::now();
                self.ogl.read();
                println!("Read: {:?}", time.elapsed());

                let time = Instant::now();
                let ptr: *mut c_void = pbo.map();
                println!("Map: {:?}", time.elapsed());
                
                let time = Instant::now();
                self.ogl.export_mp4(unsafe { from_raw_parts(ptr as *const u8, self.ogl.bytes) });
                println!("Export: {:?}", time.elapsed());
                
                pbo.unmap();
                
                self.ogl.frame += 1;
                let name: String = format!("temp/{:010}.mp4", self.ogl.frame);
                let filename: &str = name.as_str();
                writeln!(index, "file {}", filename).unwrap();
                
                // self.handles.push(spawn(move ||{
                //     ogl.export_mp4();
                //     ogl
                // }));
                
                // }
        }
        concat_mp4(&self.params.mp4_file.clone()); // ≃1/4 of runtime
        
        Ok(())
    }

    pub fn full_png(&mut self) -> Result<(), String> {
        // let ogl = self.handles.remove(0).join().unwrap();
        
        self.ogl.to_zero();
        for y in self.ogl.notes.vert
        .iter_mut()
        .skip(1)
        .step_by(3) 
        {
            *y=(*y/(self.ogl.max_frame as f32/self.ogl.framerate)-0.5)*2.;
        }
        
        unsafe { gl::Uniform1f(self.ogl.u_time.id, 0.0); }
        // unsafe { gl::Viewport(0, 0, (self.ogl.width/4) as i32, (self.ogl.height*3) as i32); } // with framebuffer change as well
        self.ogl.draw();
        self.ogl.read();
        let png_file = self.params.png_file.clone();
        // spawn(move ||{
        self.ogl.export_png(&png_file);
        println!("✨ Generated an image of the full song! ✨");
            // self.ogl.frame += self.ogl.cores;
        // });
        // self.ogl = OpenGLContext::new(self.params.width, self.params.height, self.params.framerate, self.params.cores, &self.params.midi_file);

        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }
}
