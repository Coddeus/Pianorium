extern crate egui_sdl2_gl;
// extern crate ffmpeg_next as ffmpeg;
extern crate midly;
extern crate num_cpus;
extern crate rand;

fn main() {
    // ffmpeg::init().unwrap();

    let mut p = Pianorium::new().unwrap();
    p.play().unwrap();
    p.full_mp4().unwrap();
    // p.full_png().unwrap();
}

use egui_sdl2_gl::{
    egui::{self, color::Hsva, color_picker::Alpha, epaint, style, Color32, CtxRef},
    gl::{self, types::*},
    painter::Painter,
    sdl2::{
        self,
        event::Event,
        video::{GLContext, SwapInterval, Window},
        EventPump, Sdl,
    },
    with_sdl2, DpiScaling, EguiStateHandler, ShaderVersion,
};

use std::{
    f32::consts::PI,
    ffi::{c_void, CStr, CString},
    fs::{create_dir, remove_dir_all, remove_file, File},
    io::{Read, Write},
    process::{Command, Stdio},
    ptr::null,
    slice::from_raw_parts,
    time::Instant,
};

use midly::{
    num::{u15, u24, u28, u7},
    MetaMessage::{EndOfTrack, Tempo},
    MidiMessage::{NoteOff, NoteOn},
    Smf,
    Timing::{Metrical, Timecode},
    TrackEventKind::Meta,
    TrackEventKind::Midi,
};

use rand::{thread_rng, Rng};


pub const LAYOUT: [[f32; 2]; 88] = [
    [-26. / 26., -25. / 26.],
    [-25.15 / 26., -24.55 / 26.],
    [-25. / 26., -24. / 26.],
    [-24. / 26., -23. / 26.],
    [-23.4 / 26., -22.8 / 26.],
    [-23. / 26., -22. / 26.],
    [-22.2 / 26., -21.6 / 26.],
    [-22. / 26., -21. / 26.],
    [-21. / 26., -20. / 26.],
    [-20.45 / 26., -19.85 / 26.],
    [-20. / 26., -19. / 26.],
    [-19.3 / 26., -18.7 / 26.],
    [-19. / 26., -18. / 26.],
    [-18.15 / 26., -17.55 / 26.],
    [-18. / 26., -17. / 26.],
    [-17. / 26., -16. / 26.],
    [-16.4 / 26., -15.8 / 26.],
    [-16. / 26., -15. / 26.],
    [-15.2 / 26., -14.6 / 26.],
    [-15. / 26., -14. / 26.],
    [-14. / 26., -13. / 26.],
    [-13.45 / 26., -12.85 / 26.],
    [-13. / 26., -12. / 26.],
    [-12.3 / 26., -11.7 / 26.],
    [-12. / 26., -11. / 26.],
    [-11.15 / 26., -10.55 / 26.],
    [-11. / 26., -10. / 26.],
    [-10. / 26., -9. / 26.],
    [-9.4 / 26., -8.8 / 26.],
    [-9. / 26., -8. / 26.],
    [-8.2 / 26., -7.6 / 26.],
    [-8. / 26., -7. / 26.],
    [-7. / 26., -6. / 26.],
    [-6.45 / 26., -5.85 / 26.],
    [-6. / 26., -5. / 26.],
    [-5.3 / 26., -4.7 / 26.],
    [-5. / 26., -4. / 26.],
    [-4.15 / 26., -3.55 / 26.],
    [-4. / 26., -3. / 26.],
    [-3. / 26., -2. / 26.],
    [-2.4 / 26., -1.8 / 26.],
    [-2. / 26., -1. / 26.],
    [-1.2 / 26., -0.6 / 26.],
    [-1. / 26., 0. / 26.],
    [0. / 26., 1. / 26.],
    [0.55 / 26., 1.15 / 26.],
    [1. / 26., 2. / 26.],
    [1.7 / 26., 2.3 / 26.],
    [2. / 26., 3. / 26.],
    [2.85 / 26., 3.45 / 26.],
    [3. / 26., 4. / 26.],
    [4. / 26., 5. / 26.],
    [4.6 / 26., 5.2 / 26.],
    [5. / 26., 6. / 26.],
    [5.8 / 26., 6.4 / 26.],
    [6. / 26., 7. / 26.],
    [7. / 26., 8. / 26.],
    [7.55 / 26., 8.15 / 26.],
    [8. / 26., 9. / 26.],
    [8.7 / 26., 9.3 / 26.],
    [9. / 26., 10. / 26.],
    [9.85 / 26., 10.45 / 26.],
    [10. / 26., 11. / 26.],
    [11. / 26., 12. / 26.],
    [11.6 / 26., 12.2 / 26.],
    [12. / 26., 13. / 26.],
    [12.8 / 26., 13.4 / 26.],
    [13. / 26., 14. / 26.],
    [14. / 26., 15. / 26.],
    [14.55 / 26., 15.15 / 26.],
    [15. / 26., 16. / 26.],
    [15.7 / 26., 16.3 / 26.],
    [16. / 26., 17. / 26.],
    [16.85 / 26., 17.45 / 26.],
    [17. / 26., 18. / 26.],
    [18. / 26., 19. / 26.],
    [18.6 / 26., 19.2 / 26.],
    [19. / 26., 20. / 26.],
    [19.8 / 26., 20.4 / 26.],
    [20. / 26., 21. / 26.],
    [21. / 26., 22. / 26.],
    [21.55 / 26., 22.15 / 26.],
    [22. / 26., 23. / 26.],
    [22.7 / 26., 23.3 / 26.],
    [23. / 26., 24. / 26.],
    [23.85 / 26., 24.45 / 26.],
    [24. / 26., 25. / 26.],
    [25. / 26., 26. / 26.],
]; // Look for LAYOUT[midinote-21]

pub const BLACK: [u8; 36] = [
    1, 4, 6, 9, 11, 13, 16, 18, 21, 23, 25, 28, 30, 33, 35, 37, 40, 42, 45, 47, 49, 52, 54, 57, 59,
    61, 64, 66, 69, 71, 73, 76, 78, 81, 83, 85,
];

/// The full application.
pub struct Pianorium {
    /// The rendering parameters, which can be changed through the GUI.
    pub p: Parameters,
    /// What the user sees: a window, an OpenGL context, a GUI.
    pub display: Display,
    /// The background task which renders the final video.
    pub renderer: Renderer,
}

impl Drop for Pianorium {
    fn drop(&mut self) {
        if self.p.clear_dir {
            Self::teardown().unwrap();
        }
    }
}

impl Pianorium {
    /// Creates a ready-to-use Pianorium app.
    pub fn new() -> Result<Self, &'static str> {
        let mut p: Parameters = Parameters::default();

        let display: Display = {
            let winsdl: Winsdl = Winsdl::new(800, 600, 3).unwrap();
            // HANDLES FOR OPENGL
            // let handles: Vec<JoinHandle<OpenGLContext>> = fill_handles(p.width, p.height, p.framerate, &p.midi_file).unwrap();
            let (ogl, max_frame) = OpenGLContext::new(20, 20, 60.0, &p.midi_file);
            p.max_frame = max_frame;
            let gui: Gui = Gui::new(&winsdl.window).unwrap();

            Display {
                winsdl,
                ogl,
                gui
            }
        };

        let renderer: Renderer = {
            // HANDLES FOR OPENGL
            // let handles: Vec<JoinHandle<OpenGLContext>> = fill_handles(p.width, p.height, p.framerate, &p.midi_file).unwrap();
            let (ogl, _) = OpenGLContext::new(800, 600, 60.0, &p.midi_file);
            let encoder: Encoder = Encoder {};

            Renderer {
                ogl,
                encoder
            }
        };

        Self::setup().unwrap();

        Ok(Pianorium {
            p,
            display,
            renderer,
        })
    }
    /// Records the whole song in the background.
    pub fn full_mp4(&mut self) -> Result<(), String> {
        self.renderer.ogl.to_zero(self.p.speed / self.p.cores as f32 * self.renderer.ogl.frame as f32);
        self.display.winsdl
            .window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();

        let mut index = File::create(self.p.index_file.clone()).unwrap();
        println!("Rendering frames…");

        self.renderer.ogl.vbo.set(&self.renderer.ogl.notes.vert);
        self.renderer.ogl.vao.set();
        self.renderer.ogl.ibo.set(&self.renderer.ogl.notes.ind);
        self.renderer.ogl.program.set_used();

        let tex = Texture::gen();
        tex.set(self.p.width as i32, self.p.height as i32);
        let fbo = Fbo::gen();
        fbo.set(tex.id);

        let pbo = Pbo::gen();
        pbo.set(self.p.bytes);

        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        }

        'record: loop {
            for event in self.display.winsdl.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'record,
                    _ => {} // egui_state.process_input(&window, event, &mut painter);
                }
            }

            // for _u in 0..self.p.cores {
            // let mut ogl = self.handles.remove(0).join().unwrap();
            if self.renderer.ogl.frame > self.p.max_frame {
                break 'record;
            } // Stop when it's finished playing
            unsafe {
                gl::Uniform1f(self.renderer.ogl.u_time.id, self.renderer.ogl.frame as f32 / self.p.framerate);
            }

            self.renderer.ogl.update(1.0 / self.p.framerate);
            self.renderer.ogl.draw();

            let time = Instant::now();
            self.read();
            println!("Read: {:?}", time.elapsed());

            let time = Instant::now();
            let ptr: *mut c_void = pbo.map();
            println!("Map: {:?}", time.elapsed());

            let time = Instant::now();
            self.export_mp4(unsafe { from_raw_parts(ptr as *const u8, self.p.bytes) });
            println!("Export: {:?}", time.elapsed());

            pbo.unmap();

            self.renderer.ogl.frame += 1;
            let name: String = format!("temp/{:010}.mp4", self.renderer.ogl.frame);
            let filename: &str = name.as_str();
            writeln!(index, "file {}", filename).unwrap();

            // self.handles.push(spawn(move ||{
            //     ogl.export_mp4();
            //     ogl
            // }));

            // }
        }
        Self::concat_mp4(&self.p.mp4_file.clone()); // ≃1/4 of runtime

        Ok(())
    }

    /// Saves an image of the full song.
    pub fn full_png(&mut self) -> Result<(), String> {
        // let ogl = self.handles.remove(0).join().unwrap();

        self.renderer.ogl.to_zero(self.p.speed / self.p.cores as f32 * self.renderer.ogl.frame as f32);
        for y in self.renderer.ogl.notes.vert.iter_mut().skip(1).step_by(3) {
            *y = (*y / (self.p.max_frame as f32 / self.p.framerate) - 0.5) * 2.;
        }

        unsafe {
            gl::Uniform1f(self.renderer.ogl.u_time.id, 0.0);
        }
        // unsafe { gl::Viewport(0, 0, (self.renderer.ogl.width/4) as i32, (self.renderer.ogl.height*3) as i32); } // with framebuffer change as well
        self.renderer.ogl.draw();
        self.read();
        let png_file = self.p.png_file.clone();
        // spawn(move ||{
        self.export_png(&png_file);
        println!("✨ Generated an image of the full song! ✨");
        // self.renderer.ogl.frame += self.renderer.ogl.cores;
        // });
        // self.renderer.ogl = OpenGLContext::new(self.p.width, self.p.height, self.p.framerate, self.p.cores, &self.p.midi_file);

        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }

    /// Plays the song with realtime changes from the GUI.
    pub fn play(&mut self) -> Result<(), String> {
        self.display.ogl.to_zero(self.p.speed / self.p.cores as f32 * self.display.ogl.frame as f32);
        self.display.winsdl
            .window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();

        // let ogl = self.handles.remove(0).join().unwrap();
        // self.ogl.to_zero();
        let mut rgb: [f32; 3] = [0.1, 0.1, 0.1];

        println!("✨ Playing the visualization ✨");
        let start_time = Instant::now();
        let mut since_last: f32;
        let mut since_start: f32 = 0.0;
        'play: loop {
            since_last = start_time.elapsed().as_secs_f32() - since_start;
            since_start += since_last;

            if self.display.ogl.frame > self.p.max_frame {
                break 'play;
            } // Stop when it's finished playing

            self.display.gui.egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
            self.display.gui
                .egui_ctx
                .begin_frame(self.display.gui.egui_state.input.take());

            self.display.ogl.vbo.set(&self.display.ogl.notes.vert);
            self.display.ogl.vao.set();
            self.display.ogl.ibo.set(&self.display.ogl.notes.ind);
            self.display.ogl.program.set_used();
            unsafe {
                gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
                gl::Uniform1f(self.display.ogl.u_time.id, since_start as f32);
            }
            self.display.ogl.update(since_last);
            self.display.ogl.draw();
            self.display.ogl.frame += 1;

            self.draw_gui();
            rgb = self.p.bg.to_rgb();

            let (egui_output, paint_cmds) = self.display.gui.egui_ctx.end_frame();
            self.display.gui
                .egui_state
                .process_output(&self.display.winsdl.window, &egui_output);
            let paint_jobs = self.display.gui.egui_ctx.tessellate(paint_cmds);
            self.display.gui
                .painter
                .paint_jobs(None, paint_jobs, &self.display.gui.egui_ctx.font_image());

            self.display.winsdl.window.gl_swap_window();

            // if !egui_output.needs_repaint {
            //     if let Some(event) = self.display.winsdl.event_pump.wait_event_timeout(5) {
            //         match event {
            //             Event::Quit { .. } => break 'play,
            //             _ => {
            //                 // Process input event
            //                 self.display.gui.egui_state.process_input(&self.display.winsdl.window, event, &mut self.display.gui.painter);
            //             }
            //         }
            //     }
            // } else {
            for event in self.display.winsdl.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'play,
                    _ => {
                        // Process input event
                        self.display.gui.egui_state.process_input(
                            &self.display.winsdl.window,
                            event,
                            &mut self.display.gui.painter,
                        );
                    }
                }
            }
            // }
        }
        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }

    /// Draws the GUI.
    fn draw_gui(&mut self) {
        egui::Window::new("Pianorium").show(&self.display.gui.egui_ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.bg,
                    self.p.alpha,
                );
                ui.label("Background color");
            });
            ui.horizontal(|ui| {
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.notes,
                    self.p.alpha,
                );
                ui.label("Notes color");
            });
            ui.horizontal(|ui| {
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particles,
                    self.p.alpha,
                );
                ui.label("Particles color");
            });
        });
    }

    fn setup() -> std::io::Result<()> {
        let _ = remove_dir_all("temp");
        create_dir("temp")?;
        Ok(())
    }
    
    fn teardown() -> std::io::Result<()> {
        remove_dir_all("temp")?;
        let _ = remove_file("index.txt");
        let _ = remove_file("ff_concat_mp4.log");
        let _ = remove_file("ff_export_mp4.log");
        let _ = remove_file("ff_export_png.log");
        Ok(())
    }

    pub fn read(&mut self) {
        unsafe {
            gl::ReadPixels(
                0,
                0,
                self.p.width as i32,
                self.p.height as i32,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                null::<u8>() as *mut gl::types::GLvoid,
            );
        }
    }

    pub fn export_mp4(&self, ptr: &[u8]) {
        let name = format!("temp/{:010}.mp4", self.renderer.ogl.frame);
        let filename = name.as_str();

        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=ff_export_mp4.log:level=56")
            .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("rawvideo")
            .arg("-r")
            .arg(self.p.framerate.to_string())
            .arg("-pix_fmt")
            .arg("bgra")
            .arg("-s")
            .arg(format!("{}x{}", self.p.width, self.p.height))
            .arg("-i")
            .arg("-")
            .arg("-vcodec")
            .arg("libx264")
            .arg("-crf")
            .arg("23")
            .arg("-vf")
            .arg("vflip")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(ptr).unwrap();
        }
    }

    pub fn export_png(&self, filename: &str) {
        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=ff_export_png.log:level=56")
            .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("bgra")
            .arg("-s")
            .arg(format!("{}x{}", self.p.width, self.p.height))
            .arg("-i")
            .arg("-")
            .arg("-frames:v")
            .arg("1")
            .arg("-vf")
            .arg("vflip")
            .arg("-y")
            .arg(filename)
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg process.");

        if let Some(ref mut stdin) = ffmpeg.stdin {
            stdin.write_all(&self.renderer.ogl.data).unwrap();
        }
    }

    pub fn concat_mp4(output: &str) {
        println!("Concatenating into one video…");

        Command::new("ffmpeg")
            .env("FFREPORT", "file=ff_concat_mp4.log:level=56")
            .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("concat")
            .arg("-i")
            .arg("index.txt")
            .arg("-c")
            .arg("copy")
            .arg("-movflags")
            .arg("faststart")
            .arg("-y")
            .arg(output)
            .output()
            .unwrap();

        println!("✨ Fresh video generated! ✨");
    }
}

// pub duration: f64,
// pub tempo: f32,
// pub border_radius: u8,
// pub bgimg_file: &'static str,
// pub vertical_img_divide: u8,
pub struct Parameters {
    pub width: usize,
    pub height: usize,
    pub bytes: usize,
    pub cores: usize,
    pub samples: u8,
    pub framerate: f32,
    pub max_frame: usize,
    pub speed: f32,
    // Relative path from where the executable is called
    pub midi_file: String,
    // Relative path from where the executable is called
    pub index_file: String,
    pub mp4_file: String,
    pub png_file: String,
    pub clear_dir: bool,
    pub bg: Hsva,
    pub alpha: Alpha,
    pub notes: Hsva,
    pub particles: Hsva,
}

impl Default for Parameters {
    fn default() -> Self {
        let width: usize = 1920;
        let height: usize = 1080;
        let bytes: usize = width*height*4;
        let cores: usize = num_cpus::get();
        let samples: u8 = 11;
        let framerate: f32 = 60.0;
        let max_frame: usize = 0;
        let speed: f32 = cores as f32 / framerate;
        let midi_file = "test.mid".to_owned();
        let mp4_file = "output.mp4".to_owned();
        let png_file = "output.png".to_owned();
        let clear_dir: bool = true;
        let index_file = "index.txt".to_owned();
        let bg: Hsva = Hsva {
            h: 0.0,
            s: 0.0,
            v: 0.1,
            a: 1.0,
        };
        let alpha: Alpha = Alpha::Opaque;
        let notes: Hsva = Hsva {
            h: 0.5,
            s: 0.1,
            v: 0.1,
            a: 1.0,
        };
        let particles: Hsva = Hsva {
            h: 0.75,
            s: 0.5,
            v: 0.5,
            a: 1.0,
        };
        Parameters {
            width,
            height,
            bytes,
            cores,
            samples,
            framerate,
            max_frame,
            speed,
            midi_file,
            mp4_file,
            png_file,
            clear_dir,
            index_file,
            bg,
            alpha,
            notes,
            particles,
        }
    }
}

pub struct Display {
    /// Resizeable window
    pub winsdl: Winsdl,
    /// The OpenGL context for realtime changes
    pub ogl: OpenGLContext,
    /// GUI-drawing components
    pub gui: Gui,
}

pub struct Renderer {
    /// The OpenGL context of the rendering
    pub ogl: OpenGLContext,
    /// The FFmpeg encoder, converts framebuffer data to video frames
    pub encoder: Encoder,
}

pub struct Encoder {}

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


// use std::sync::{Arc, Mutex};

pub struct OpenGLContext {
    pub frame: usize,

    pub data: Vec<u8>,

    pub notes: Notes,
    pub particles: Particles,

    pub vbo: Vbo,
    pub vao: Vao,
    pub ibo: Ibo,

    pub program: Program,
    pub u_time: Uniform,
    pub u_resolution: Uniform,
}

// pub struct Shared { // Read-only
//
// }
//
// impl std::fmt::Debug for OpenGLContext {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(f, "ogl{}", self.shared.frame)
//     }
// }
//
// impl Clone for OpenGLContext{
//     fn clone(&self) -> OpenGLContext {
//
//         OpenGLContext {
//             width: self.width,
//             height: self.height,
//             bytes: self.bytes,
//             cores: self.cores,
//             framerate: self.framerate,
//             speed: self.speed,
//             max_frame: self.max_frame,
//
//             data: self.data.clone(),
//
//             shared: self.shared.clone(),
//         }
//             .setup_vbo()
//             .setup_vao()
//             .setup_context()
//             .setup_ibo()
//

// impl Clone for OpenGLContext{
//     fn clone(&self) -> OpenGLContext {
//         OpenGLContext {
//             width: self.width,
//             height: self.height,
//             bytes: self.bytes,
//             cores: self.cores,
//
//             frame: self.frame,
//             framerate: self.framerate,
//             speed: self.speed,
//             max_frame: self.max_frame,
//
//             data: self.data.clone(),
//
//             notes: self.notes.clone(),
//             particles: self.particles.clone(),
//
//             vbo: self.vbo,
//             vao: self.vao,
//             ibo: self.ibo,
//
//             program: self.program.clone(),
//             u_time: self.u_time.clone(),
//             u_resolution: self.u_resolution.clone(),
//         }
//             .setup_vbo()
//             .setup_vao()
//             .setup_context()
//             .setup_ibo()
//     }
// }

impl OpenGLContext {
    /// Returns a ready-to-use context and the final frame number
    pub fn new(width: usize, height: usize, framerate: f32, midi_file: &str) -> (Self, usize) {
        let bytes: usize = width * height * 4;
        let data: Vec<u8> = vec![0; bytes];
        
        let frame: usize = 0;
        let (notes, max_frame) =
            Notes::from_midi(width as f32 / height as f32, framerate, midi_file).unwrap();

        let particles: Particles = Particles::new();

        let program: Program = create_program().unwrap();
        let u_time: Uniform = Uniform::new(program.id, "u_time").unwrap();
        let u_resolution: Uniform = Uniform::new(program.id, "u_resolution").unwrap();

        unsafe {
            gl::Uniform1f(u_time.id, 0.0);
            gl::Uniform2f(u_resolution.id, width as f32, height as f32);
        }

        let vbo: Vbo = Vbo::gen();
        vbo.set(&notes.vert);
        let vao: Vao = Vao::gen();
        vao.set();
        let ibo: Ibo = Ibo::gen();
        ibo.set(&notes.ind);

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }

        (OpenGLContext {
            frame,

            data,

            notes,
            particles,

            vbo,
            vao,
            ibo,

            program,
            u_time,
            u_resolution,
        }, max_frame)
    }

    pub fn to_zero(&mut self, units: f32) {
        self.frame = 0;
        self.particles = Particles::new();
        self.update(-units);
    }

// pub fn fill_handles(width: usize, height: usize, framerate: f32, cores: usize, midi_file: &str) -> Result<Vec<std::thread::JoinHandle<OpenGLContext>>, &'static str> {
//     let mut ogls: Vec<OpenGLContext> = vec![OpenGLContext::new(width, height, framerate, cores, midi_file)];
//     for _u in 1..cores {
//         ogls.push(ogls[ogls.len()-1].clone());
//     }
//
//     let mut handles: Vec<std::thread::JoinHandle<OpenGLContext>> = vec![];
//     for _u in 0..cores {
//         let ogl = ogls.remove(0);
//         handles.push(std::thread::spawn(move || {ogl}));
//     }
//     Ok(handles)
// }

    pub fn draw(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            self.draw_notes();
            self.draw_particles();
        }
    }

    pub fn draw_notes(&mut self) {
        unsafe {
            self.vbo.set(&self.notes.vert);
            self.ibo.set(&self.notes.ind);
            gl::DrawElements(
                gl::TRIANGLES,
                self.notes.ind.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
        }
    }

    pub fn draw_particles(&mut self) {
        unsafe {
            self.vbo.set(&self.particles.vert);
            self.ibo.set(&self.particles.ind);
            gl::DrawElements(
                gl::TRIANGLES,
                self.particles.ind.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
        }
    }

    pub fn update(&mut self, diff: f32) {
        for y in self.notes.vert.iter_mut().skip(1).step_by(3) {
            *y -= diff;
        }

        self.particles.update(diff, &self.notes.vert);
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub note: u8, // A0 is 21 ; C8 is 108
    pub start: f32,
    pub end: f32,
}

#[derive(Clone)]
pub struct Notes {
    pub notes: Vec<Note>,
    pub vert: Vec<f32>,
    pub ind: Vec<u32>,
}

impl Notes {
    pub fn from_midi(
        wh_ratio: f32,
        framerate: f32,
        midi_file: &str,
    ) -> std::io::Result<(Notes, usize)> {
        // Done Twice instead of just ….clone().iter_mut { +0.5 }
        let mut notes: Vec<Note> = vec![];
        let mut blacknotes: Vec<Note> = vec![];
        let mut active_notes: Vec<Option<Note>> = vec![None; 128];

        let mut file = File::open(midi_file)
            .expect("\nMidi file could not be opened. \nCheck the file path, and retry");
        let mut buf: Vec<u8> = vec![];
        let numbytes: usize = file
            .read_to_end(&mut buf)
            .expect("\nMidi file could not be read.");
        println!("Reading {}-byte midi file…", numbytes);
        let midi_data = Smf::parse(&buf).unwrap();

        let mut spb: f32 = 0.5; // Seconds per tick
        let mut spt: f32; // Seconds per beat
        match midi_data.header.timing {
            Metrical(m) => {
                let ppq: f32 = <u15 as Into<u16>>::into(m) as f32;
                spt = spb / ppq;
            }
            Timecode(fps, sfpf) => {
                spt = 1. / fps.as_f32() / sfpf as f32;
            }
        }
        let mut max_frame: usize = 0;

        for track in midi_data.tracks.iter() {
            let mut current_time: f32 = 2.;
            for event in track.iter() {
                current_time += <u28 as Into<u32>>::into(event.delta) as f32 * spt;
                match event.kind {
                    Midi {
                        channel: _,
                        message,
                    } => match message {
                        NoteOn { key, vel } => {
                            if 20 < key && key < 109 {
                                if vel > 0 {
                                    let note_obj = Note {
                                        note: <u7 as Into<u8>>::into(key),
                                        start: current_time,
                                        end: 0.,
                                    };
                                    active_notes[<u7 as Into<u8>>::into(key) as usize] =
                                        Some(note_obj);
                                } else {
                                    if let Some(mut note_obj) =
                                        active_notes[<u7 as Into<u8>>::into(key) as usize].take()
                                    {
                                        note_obj.end = current_time;
                                        if BLACK.contains(&note_obj.note) {
                                            blacknotes.push(note_obj);
                                        } else {
                                            notes.push(note_obj);
                                        }
                                        active_notes[<u7 as Into<u8>>::into(key) as usize] = None;
                                    }
                                }
                            }
                        }
                        NoteOff { key, vel: _ } => {
                            if let Some(mut note_obj) =
                                active_notes[<u7 as Into<u8>>::into(key) as usize].take()
                            {
                                note_obj.end = current_time;
                                if BLACK.contains(&note_obj.note) {
                                    blacknotes.push(note_obj);
                                } else {
                                    notes.push(note_obj);
                                }
                                active_notes[<u7 as Into<u8>>::into(key) as usize] = None;
                            }
                        }
                        _ => {}
                    },

                    Meta(message) => {
                        match message {
                            Tempo(t) => {
                                // This event should only be present when header timing is "Metrical"
                                let tempo: f32 = <u24 as Into<u32>>::into(t) as f32 / 1000000.;
                                spt = spt / spb * tempo;
                                spb = tempo;
                            }
                            EndOfTrack => {
                                // Know when the render finishes
                                max_frame = ((current_time + 4.) * framerate) as usize;
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }

        notes.extend(blacknotes);

        let mut new = Notes {
            notes,
            vert: vec![],
            ind: vec![],
        };
        new.notes_to_vertices(wh_ratio).unwrap();

        Ok((new, max_frame))
    }

    pub fn notes_to_vertices(&mut self, wh_ratio: f32) -> std::io::Result<()> {
        for (i, n) in self.notes.iter().enumerate() {
            let ver2: Vec<f32> = vec![
                //               x                             y          color
                LAYOUT[n.note as usize - 21][0],
                (n.start),
                1.0,
                LAYOUT[n.note as usize - 21][1],
                (n.start),
                1.0,
                LAYOUT[n.note as usize - 21][1],
                (n.end),
                1.0,
                LAYOUT[n.note as usize - 21][0],
                (n.end),
                1.0,
                //               x                             y          color
                LAYOUT[n.note as usize - 21][0] + 0.007,
                (n.start + 0.007 * wh_ratio),
                0.9,
                LAYOUT[n.note as usize - 21][1] - 0.007,
                (n.start + 0.007 * wh_ratio),
                0.9,
                LAYOUT[n.note as usize - 21][1] - 0.007,
                (n.end - 0.007 * wh_ratio),
                0.9,
                LAYOUT[n.note as usize - 21][0] + 0.007,
                (n.end - 0.007 * wh_ratio),
                0.9,
            ];
            self.vert.extend(ver2);

            let i2: u32 = i as u32;
            let ind2: Vec<u32> = vec![
                0 + 8 * i2,
                2 + 8 * i2,
                1 + 8 * i2,
                0 + 8 * i2,
                2 + 8 * i2,
                3 + 8 * i2,
                4 + 8 * i2,
                6 + 8 * i2,
                5 + 8 * i2,
                4 + 8 * i2,
                6 + 8 * i2,
                7 + 8 * i2,
            ];
            self.ind.extend(ind2);
        }

        Ok(())
    }
}

pub struct Fbo {
    pub id: GLuint,
}

impl Drop for Fbo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Fbo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }
        Fbo { id }
    }

    pub fn set(&self, texture: GLuint) {
        self.bind();
        self.tex(texture);
    }

    fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
        self.check()
    }

    fn check(&self) {
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            println!(
                "🛑 Framebuffer wasn't successfully bound. Error {:#?}",
                status
            );
        }
    }

    fn tex(&self, texture: GLuint) {
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );
        }
    }

    fn unbind(&self) {
        // Back to default window-projected framebuffer
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

pub struct Ibo {
    pub id: GLuint,
}

impl Drop for Ibo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Ibo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Ibo { id }
    }

    pub fn set(&self, data: &Vec<u32>) {
        self.bind();
        self.data(data);
    }

    fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    fn data(&self, indices: &Vec<u32>) {
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct Pbo {
    pub id: GLuint,
}

impl Drop for Pbo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Pbo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Pbo { id }
    }

    pub fn set(&self, bytes: usize) {
        self.bind();
        self.data(bytes);
    }

    fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, self.id);
        }
    }

    fn data(&self, bytes: usize) {
        unsafe {
            gl::BufferData(
                gl::PIXEL_PACK_BUFFER,
                bytes as gl::types::GLsizeiptr,
                null() as *const gl::types::GLvoid,
                gl::STREAM_READ,
            );
        }
    }

    pub fn map(&self) -> *mut GLvoid {
        unsafe { gl::MapBuffer(gl::PIXEL_PACK_BUFFER, gl::READ_ONLY) }
    }

    pub fn unmap(&self) -> u8 {
        unsafe { gl::UnmapBuffer(gl::PIXEL_PACK_BUFFER) }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct Texture {
    pub id: GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Texture {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Texture { id }
    }

    pub fn set(&self, width: i32, height: i32) {
        self.bind();
        self.setup(width, height);
    }

    fn setup(&self, width: i32, height: i32) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::BGRA as i32,
                width,
                height,
                0,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                null(),
            );
        }
    }

    fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub struct Vao {
    pub id: GLuint,
}

impl Drop for Vao {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Vao {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        Vao { id }
    }

    pub fn set(&self) {
        self.bind();
        self.setup();
    }

    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    fn setup(&self) {
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                1,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct Vbo {
    pub id: GLuint,
}

impl Drop for Vbo {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Vbo {
    pub fn gen() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Vbo { id }
    }

    pub fn set(&self, data: &Vec<f32>) {
        self.bind();
        self.data(data);
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    fn data(&self, vertices: &Vec<f32>) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub position: (f32, f32),
    pub direction: (f32, f32),
    pub lifetime: f32,
}

impl Particle {
    fn new(x: f32, seed: f32) -> Self {
        let mut rng = thread_rng();
        let r = rng.gen_range(-1.0..1.0);
        Particle {
            position: (x, -1.),
            direction: (
                seed / 20. + r / 20.,
                rng.gen_range(0.3 + 0.15 * r.abs()..(0.7 - 0.2 * r.abs())),
            ),
            lifetime: 1.5 - (r * PI / 2.).sin().abs() / 3.,
        }
    }

    fn update(&mut self, elapsed: f32) {
        self.position.0 += self.direction.0 * elapsed;
        self.position.1 += self.direction.1 * elapsed;
        self.lifetime -= elapsed;
    }
}

#[derive(Clone, Debug)]
pub struct Particles {
    pub particles: Vec<Particle>,
    pub vert: Vec<f32>,
    pub ind: Vec<u32>,
}

impl Particles {
    pub fn new() -> Self {
        Particles {
            particles: vec![],
            vert: vec![],
            ind: vec![],
        }
    }

    pub fn update(&mut self, elapsed: f32, note_vert: &Vec<f32>) {
        for p in self.particles.iter_mut() {
            p.update(elapsed)
        }
        self.particles.retain(|p| p.lifetime > 0.);

        let mut i: usize = 0;
        while i < note_vert.len() {
            if note_vert[i + 1] < (-1.) && note_vert[i + 7] > (-1.) {
                for _ in 0..(elapsed * 3000.) as usize {
                    self.particles.push(Particle::new(
                        (note_vert[i] + note_vert[i + 6]) / 2.,
                        (1000. * note_vert[i]).sin(),
                    )); // (self.particles.len() as f32).sin()
                }
            }
            i += 24;
        }

        self.particles_to_vertices().unwrap();
    }

    pub fn particles_to_vertices(&mut self) -> std::io::Result<()> {
        self.vert = vec![];
        self.ind = vec![];
        for (i, p) in self.particles.iter().enumerate() {
            let ver2: Vec<f32> = vec![
                //      x                 y        color
                p.position.0 - 0.0007,
                p.position.1 - 0.0007,
                0.8,
                p.position.0 + 0.0007,
                p.position.1 - 0.0007,
                0.8,
                p.position.0 + 0.0007,
                p.position.1 + 0.0007,
                0.8,
                p.position.0 - 0.0007,
                p.position.1 + 0.0007,
                0.8,
            ];
            self.vert.extend(ver2);

            let i2: u32 = i as u32;
            let ind2: Vec<u32> = vec![
                0 + 4 * i2,
                2 + 4 * i2,
                1 + 4 * i2,
                0 + 4 * i2,
                2 + 4 * i2,
                3 + 4 * i2,
            ];
            self.ind.extend(ind2);
        }
        Ok(())
    }
}

pub struct Program {
    pub id: GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(program_id);
        }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id());
            }
        }

        Ok(Program { id: program_id })
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn from_source(source: &CStr, kind: GLenum) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn shader_from_source(source: &CStr, kind: GLenum) -> Result<GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), null());
        gl::CompileShader(id);
    }

    let mut success: GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn create_program() -> Result<Program, &'static str> {
    let vert_shader =
        Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

    let frag_shader =
        Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();

    let shader_program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    shader_program.set_used();

    Ok(Program {
        id: shader_program.id,
    })
}

pub struct Uniform {
    pub id: GLint,
}

impl Uniform {
    pub fn new(shader: u32, name: &str) -> Result<Self, &'static str> {
        let cname: CString = CString::new(name).expect("CString::new failed");
        let location: GLint;
        unsafe {
            location = gl::GetUniformLocation(shader, cname.as_ptr());
        }
        Ok(Uniform { id: location })
    }
}

pub struct Gui {
    pub painter: Painter,
    pub egui_state: EguiStateHandler,
    pub egui_ctx: CtxRef,
}

impl Gui {
    pub fn new(window: &Window) -> Result<Self, &'static str> {
        let (painter, egui_state) = with_sdl2(window, ShaderVersion::Default, DpiScaling::Default);
        let egui_ctx = CtxRef::default();
        egui_set_theme(&*egui_ctx, FRAPPE);

        Ok(Gui {
            painter,
            egui_state,
            egui_ctx,
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Theme {
    pub rosewater: Color32,
    pub flamingo: Color32,
    pub pink: Color32,
    pub mauve: Color32,
    pub red: Color32,
    pub maroon: Color32,
    pub peach: Color32,
    pub yellow: Color32,
    pub green: Color32,
    pub teal: Color32,
    pub sky: Color32,
    pub sapphire: Color32,
    pub blue: Color32,
    pub lavender: Color32,
    pub text: Color32,
    pub subtext1: Color32,
    pub subtext0: Color32,
    pub overlay2: Color32,
    pub overlay1: Color32,
    pub overlay0: Color32,
    pub surface2: Color32,
    pub surface1: Color32,
    pub surface0: Color32,
    pub base: Color32,
    pub mantle: Color32,
    pub crust: Color32,
}

// Catppuccin color theme
pub const FRAPPE: Theme = Theme {
    rosewater: Color32::from_rgb(242, 213, 207),
    flamingo: Color32::from_rgb(238, 190, 190),
    pink: Color32::from_rgb(244, 184, 228),
    mauve: Color32::from_rgb(202, 158, 230),
    red: Color32::from_rgb(231, 130, 132),
    maroon: Color32::from_rgb(234, 153, 156),
    peach: Color32::from_rgb(239, 159, 118),
    yellow: Color32::from_rgb(229, 200, 144),
    green: Color32::from_rgb(166, 209, 137),
    teal: Color32::from_rgb(129, 200, 190),
    sky: Color32::from_rgb(153, 209, 219),
    sapphire: Color32::from_rgb(133, 193, 220),
    blue: Color32::from_rgb(140, 170, 238),
    lavender: Color32::from_rgb(186, 187, 241),
    text: Color32::from_rgb(198, 208, 245),
    subtext1: Color32::from_rgb(181, 191, 226),
    subtext0: Color32::from_rgb(165, 173, 206),
    overlay2: Color32::from_rgb(148, 156, 187),
    overlay1: Color32::from_rgb(131, 139, 167),
    overlay0: Color32::from_rgb(115, 121, 148),
    surface2: Color32::from_rgb(98, 104, 128),
    surface1: Color32::from_rgb(81, 87, 109),
    surface0: Color32::from_rgb(65, 69, 89),
    base: Color32::from_rgb(48, 52, 70),
    mantle: Color32::from_rgb(41, 44, 60),
    crust: Color32::from_rgb(35, 38, 52),
};

fn make_widget_visual(
    old: style::WidgetVisuals,
    theme: &Theme,
    bg_fill: egui::Color32,
) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill,
        bg_stroke: egui::Stroke {
            color: theme.overlay1,
            ..old.bg_stroke
        },
        fg_stroke: egui::Stroke {
            color: theme.text,
            ..old.fg_stroke
        },
        ..old
    }
}

pub fn egui_set_theme(ctx: &egui::Context, theme: Theme) {
    let old = ctx.style().visuals.clone();
    ctx.set_visuals(egui::Visuals {
        override_text_color: Some(theme.text),
        hyperlink_color: theme.rosewater,
        faint_bg_color: theme.surface0,
        extreme_bg_color: theme.crust,
        code_bg_color: theme.mantle,
        widgets: style::Widgets {
            noninteractive: make_widget_visual(old.widgets.noninteractive, &theme, theme.base),
            inactive: make_widget_visual(old.widgets.inactive, &theme, theme.surface0),
            hovered: make_widget_visual(old.widgets.hovered, &theme, theme.surface2),
            active: make_widget_visual(old.widgets.active, &theme, theme.surface1),
            open: make_widget_visual(old.widgets.open, &theme, theme.surface0),
        },
        selection: style::Selection {
            bg_fill: theme.blue.linear_multiply(0.2),
            stroke: egui::Stroke {
                color: theme.overlay1,
                ..old.selection.stroke
            },
        },
        window_shadow: epaint::Shadow {
            color: theme.base,
            ..old.window_shadow
        },
        popup_shadow: epaint::Shadow {
            color: theme.base,
            ..old.popup_shadow
        },
        ..old
    });
}

pub struct Winsdl {
    pub sdl: Sdl,
    pub window: Window,
    pub gl_context: GLContext,
    pub gl: (),
    pub event_pump: EventPump,
}

impl Winsdl {
    pub fn new(width: usize, height: usize, samples: u8) -> Result<Self, &'static str> {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        if samples > 1 {
            gl_attr.set_multisample_samples(samples);
        }

        let window = video_subsystem
            .window("Pianorium", width as u32, height as u32)
            .resizable()
            .opengl()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        let gl = gl::load_with(|s| {
            video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
        });

        window
            .subsystem()
            .gl_set_swap_interval(sdl2::video::SwapInterval::Immediate)
            .unwrap();

        if samples > 1 {
            unsafe {
                gl::Enable(gl::MULTISAMPLE);
            }
        }
        unsafe {
            gl::Enable(gl::BLEND);
        }

        let event_pump: sdl2::EventPump = sdl.event_pump().unwrap();

        Ok(Winsdl {
            sdl,
            window,
            gl_context,
            gl,
            event_pump,
        })
    }
}
