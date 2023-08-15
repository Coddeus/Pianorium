use std::time::Instant;

use egui_sdl2_gl::{egui::{self, color::Hsva, color_picker::Alpha}, sdl2::{video::SwapInterval, event::Event}};

use crate::Pianorium;


impl Pianorium {
    pub fn play(&mut self) -> Result<(), String> {
        self.ogl.to_zero();
        self.winsdl.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();
        
        // let ogl = self.handles.remove(0).join().unwrap();
        // self.ogl.to_zero();
        let mut rgb: [f32; 3] = [0.1, 0.1, 0.1];
        let mut hsva: Hsva = Hsva { h: 1.0, s: 0.0, v: 0.1, a: 1.0 };
        let alpha: Alpha = Alpha::Opaque;
        
        println!("✨ Playing the visualization ✨");
        let start_time = Instant::now();
        let mut since_last: f32;
        let mut since_start: f32 = 0.0;
        'play: loop {
            since_last = start_time.elapsed().as_secs_f32()-since_start;
            since_start += since_last;

            if self.ogl.frame > self.ogl.max_frame { break 'play; }                                                                    // Stop when it's finished playing
            self.ogl.update(since_last);

            self.gui.egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
            self.gui.egui_ctx.begin_frame(self.gui.egui_state.input.take());

            self.ogl.draw(rgb, since_start);
            self.ogl.frame += 1;
            println!("Drew frame {}", self.ogl.frame);

            egui::Window::new("Pianorium").show(&self.gui.egui_ctx, |ui| {
                // ui.add(egui::Slider::new(&mut hsva, 0.0..=1.0).text("Color"));
                // ui.label(" ");
                egui::widgets::color_picker::color_edit_button_hsva(ui, &mut hsva, alpha);
            });
            rgb = hsva.to_rgb();

            let (egui_output, paint_cmds) = self.gui.egui_ctx.end_frame();
            self.gui.egui_state.process_output(&self.winsdl.window, &egui_output);
            let paint_jobs = self.gui.egui_ctx.tessellate(paint_cmds);
            self.gui.painter.paint_jobs(None, paint_jobs, &self.gui.egui_ctx.font_image());

            self.winsdl.window.gl_swap_window();

            // if !egui_output.needs_repaint {
            //     if let Some(event) = self.winsdl.event_pump.wait_event_timeout(5) {
            //         match event {
            //             Event::Quit { .. } => break 'play,
            //             _ => {
            //                 // Process input event
            //                 self.gui.egui_state.process_input(&self.winsdl.window, event, &mut self.gui.painter);
            //             }
            //         }
            //     }
            // } else {
                for event in self.winsdl.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => break 'play,
                        _ => {
                            // Process input event
                            self.gui.egui_state.process_input(&self.winsdl.window, event, &mut self.gui.painter);
                        }
                    }
                }
            // }
        }
        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }
}