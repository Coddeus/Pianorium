use std::{fs::File, io::Write, thread::spawn};

use crate::{concat_mp4, Pianorium};

impl Pianorium {
    pub fn full_mp4(&mut self) {
        let mut index = File::create(self.params.index_file.clone()).unwrap();
    
        let mut event_pump: sdl2::EventPump = self.winsdl.sdl.event_pump().unwrap();
        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    _ => {  } // egui_state.process_input(&window, event, &mut painter);
                }
            }
    
            for _u in 0..self.params.cores {
                let mut ogl = self.handles.remove(0).join().unwrap();
                if ogl.frame > ogl.shared.max_frame { break 'main; }                                                                    // Stop when it's finished playing
                
                unsafe { gl::Uniform1f(ogl.shared.u_time.id, ogl.frame as f32/self.params.framerate); }  
                ogl.draw();
                ogl.read();
                self.winsdl.window.gl_swap_window();
                let name: String = format!("temp/{:010}.mp4", ogl.frame);
                let filename: &str = name.as_str();
                writeln!(index, "file {}", filename).unwrap();
                println!("Frame {} generated!", ogl.frame);

                self.handles.push(spawn(move ||{
                    ogl.export_mp4();
                    ogl.frame += ogl.shared.cores;
                    ogl
                }));
            }
        }
        
        concat_mp4(&self.params.mp4_file.clone()); // ≃1/4 of runtime
    }

    pub fn full_png(&mut self) {
        let ogl = self.handles.remove(0).join().unwrap();
        
        let mut tempclone_ogl = ogl.clone();
        let units: f32 = tempclone_ogl.shared.speed / tempclone_ogl.shared.cores as f32 * tempclone_ogl.frame as f32;
        for y in tempclone_ogl.vertices
            .iter_mut()
            .skip(1)
            .step_by(3) 
        {
            *y+=units;
            *y=(*y/(tempclone_ogl.shared.max_frame as f32/tempclone_ogl.shared.framerate)-0.5)*2.;
        }

        unsafe { gl::Uniform1f(ogl.shared.u_time.id, 0.0); }  
        tempclone_ogl.draw();
        tempclone_ogl.read();
        let png_file = self.params.png_file.clone();
        spawn(move ||{
            tempclone_ogl.export_png(&png_file);
            tempclone_ogl.frame += tempclone_ogl.shared.cores;
            tempclone_ogl
        });
        self.winsdl.window.gl_swap_window();
        println!("Generated an image of the full song!");

        self.handles.insert(0, std::thread::spawn(move ||{ ogl }));
    }
}
