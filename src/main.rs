extern crate egui_sdl2_gl;
extern crate ffmpeg_next as ffmpeg;
extern crate midly;
extern crate num_cpus;
extern crate rand;

pub mod actions;
pub mod fs;
pub mod opengl;
pub mod pianorium;
pub mod render;
pub mod ui;
pub mod window;

pub use actions::capture::*;
pub use fs::*;
pub use opengl::{context::*, drawing::*, layout::*, notes::*, objects::*, particles::*, shaders::*, uniforms::*};
pub use pianorium::*;
pub use render::*;
pub use ui::{cli::*, gui::*, theme::*};
pub use window::*;

fn main() {
    let mut p = pianorium::Pianorium::new().unwrap();
    p.play().unwrap();
    p.full_mp4().unwrap();
    // p.full_png().unwrap();
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