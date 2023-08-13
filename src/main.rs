extern crate gl;
extern crate sdl2;
// extern crate egui;
extern crate num_cpus;

pub mod actions;
pub mod fs;
pub mod opengl;
pub mod midi;
pub mod pianorium;
pub mod render;
pub mod ui;
pub mod window;

pub use actions::capture::*;
pub use fs::*;
pub use opengl::{context::*, drawing::*, shaders::*, uniforms::*};
pub use midi::*;
pub use pianorium::*;
pub use render::*;
pub use ui::*;
pub use window::*;

fn main() {
    let mut p = pianorium::Pianorium::new().unwrap();
    p.play().unwrap();
    p.full_png().unwrap();
    p.full_mp4().unwrap();
}

// fn draw_gui() { // Struct with Impl
// 
//     egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
//     egui_ctx.begin_frame(egui_state.input.take());
//     egui::CentralPanel::default().show(&egui_ctx, |ui| {
//         ui.label(" ");
//         ui.add(egui::Slider::new(&mut slider, 0.0..=50.0).text("Slider"));
//         ui.label(" ");
//     });
//     let (egui_output, paint_cmds) = egui_ctx.end_frame();
//     egui_state.process_output(&window, &egui_output);
//     let paint_jobs = egui_ctx.tessellate(paint_cmds);
//     painter.paint_jobs(None, paint_jobs, &egui_ctx.font_image());
// }