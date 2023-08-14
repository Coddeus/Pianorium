use std::time::Instant;

use egui_sdl2_gl::{sdl2::{video::SwapInterval, event::Event}, gl};

use crate::Pianorium;


impl Pianorium {
    pub fn play(&mut self) -> Result<(), String> {
        println!("Playing the visualization…");
        self.ogl.to_zero();
        self.winsdl.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();

        // let ogl = self.handles.remove(0).join().unwrap();
        // self.ogl.to_zero();
        
        let start_time = Instant::now();
        let mut since_last: f32;
        let mut since_start: f32 = 0.0;
        'play: loop {
            for event in self.winsdl.event_pump.poll_iter() {   // Note that dragging the window blocks the rendering on Windows
                match event {
                    Event::Quit { .. } => break 'play,
                    _ => {  } // egui_state.process_input(&window, event, &mut painter);
                }
            }
            since_last = start_time.elapsed().as_secs_f32()-since_start;
            since_start += since_last;

            if self.ogl.frame > self.ogl.max_frame { break 'play; }                                                                    // Stop when it's finished playing
            
            unsafe { gl::Uniform1f(self.ogl.u_time.id, since_start as f32); }  
            self.ogl.update(since_last);
            self.ogl.draw();
            self.ogl.frame += 1;
            self.winsdl.window.gl_swap_window();

        }
        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }
}