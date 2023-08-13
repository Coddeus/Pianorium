use std::time::Instant;

use egui_sdl2_gl::{sdl2::{video::SwapInterval, event::Event}, gl};

use crate::Pianorium;


impl Pianorium {
    pub fn play(&mut self) -> Result<(), String> {
        self.winsdl.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();

        let ogl = self.handles.remove(0).join().unwrap();
        let mut play_ogl = ogl.clone();
        play_ogl.to_zero();
        
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

            if play_ogl.frame > play_ogl.shared.max_frame { break 'play; }                                                                    // Stop when it's finished playing
            
            unsafe { gl::Uniform1f(play_ogl.shared.u_time.id, since_start as f32); }  
            play_ogl.draw();
            play_ogl.update(since_last);
            self.winsdl.window.gl_swap_window();

        }
        self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }
}