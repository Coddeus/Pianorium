// #![windows_subsystem = "windows"]

extern crate egui_sdl2_gl;
extern crate ffmpeg_next as ff;
extern crate midly;
extern crate native_dialog;
extern crate num_cpus;
extern crate rand;

pub mod parameters;
use ff::Rational;
pub use parameters::Parameters;
pub mod layout;
pub use layout::{BLACK, LAYOUT};
pub mod video_builder;
pub use video_builder::{VideoBuilder, video_options::VideoOptions};

use native_dialog::FileDialog;
/// Creates a new `Pianorium`, plays the chosen song, renders it, and ends.
fn main() {
    let path = FileDialog::new()
        .set_location(&std::env::current_dir().unwrap())
        .add_filter("MIDI File", &["mid"])
        .show_open_single_file()
        .unwrap();

    let path = match path {
        Some(path) => path,
        None => return,
    };
    println!("Opening midi file: {:?}", path);

    let mut pianorium = Pianorium::new().unwrap();

    pianorium.p.midi_file = path;
    (pianorium.notes, pianorium.p.max_time) =
        Notes::from_midi(800 as f32 / 600 as f32, 1.0, &pianorium.p.midi_file).unwrap();
    pianorium.vbo.set(&pianorium.notes.vert);
    pianorium.vao.set();
    pianorium.ibo.set(&pianorium.notes.ind);

    pianorium.play().unwrap();
    pianorium.full_mp4().unwrap();
    // p.full_png().unwrap();
}

use egui_sdl2_gl::{
    egui::{self, epaint, style, Color32, CtxRef},
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
    collections::HashMap,
    f32::consts::PI,
    ffi::{CStr, CString},
    fs::File,
    io::Read,
    path::PathBuf,
    ptr::{null, null_mut},
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

/// The full application.
pub struct Pianorium {
    /// The rendering parameters, which can be changed through the GUI.
    pub p: Parameters,

    pub sdl: Sdl,
    pub window: Window,
    pub gl_context: GLContext,
    pub gl: (),
    pub event_pump: EventPump,

    /// GUI-drawing components
    pub gui: Gui,

    pub frame_count: usize,
    pub ol: Ol,
    pub notes: Notes,
    pub particles: Particles,
    pub vbo: Vbo,
    pub vao: Vao,
    pub ibo: Ibo,
    // pub frames: Vec<Vec<u8>>,
}

impl Pianorium {
    /// Creates a ready-to-use Pianorium app.
    pub fn new() -> Result<Self, &'static str> {
        #[cfg(debug_assertions)]
        let time = Instant::now();

        #[cfg(target_os = "linux")]
        std::env::set_var("SDL_VIDEO_X11_VISUALID", "");

        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(16);
        let window = video_subsystem
            .window("Pianorium", 1366, 768)
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
        unsafe {
            gl::Enable(gl::MULTISAMPLE);
        }
        let event_pump: sdl2::EventPump = sdl.event_pump().unwrap();

        let p: Parameters = Parameters::default();

        let frame_count: usize = 0;

        let ol: Ol = Ol::create(p.ol_width).unwrap();
        let notes = Notes::new();
        let particles: Particles = Particles::new();

        let vbo: Vbo = Vbo::gen();
        let vao: Vao = Vao::gen();
        let ibo: Ibo = Ibo::gen();

        vbo.bind();
        ibo.bind();
        vao.set();

        let gui: Gui = Gui::new(&window).unwrap();

        #[cfg(debug_assertions)]
        println!("\nPianorium::new() in {:?}", time.elapsed());

        Ok(Pianorium {
            p,
            sdl,
            window,
            gl_context,
            gl,
            event_pump,
            gui,
            frame_count,
            ol,
            notes,
            particles,
            vbo,
            vao,
            ibo,
        })
    }

    /// Plays the song with realtime changes from the GUI.
    pub fn play(&mut self) -> Result<(), String> {
        self.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();

        let mut rgb: [f32; 3] = [0.1, 0.1, 0.1];

        #[cfg(debug_assertions)]
        println!("\n✨ Playing the visualization ✨\n");

        let start_time = Instant::now();
        let mut since_last: f32;
        let mut since_start: f32 = 0.0;
        'play: loop {
            // Loop playing
            if self.p.time > self.p.max_time {
                self.p.time -= self.p.max_time;
                self.notes.update(-self.p.max_time * self.p.gravity);
            }
            if self.p.time < 0. {
                self.p.time += self.p.max_time;
                self.notes.update(self.p.max_time * self.p.gravity);
            }

            since_last = start_time.elapsed().as_secs_f32() - since_start;
            since_start += since_last;
            let time_diff = since_last * self.p.preview_speed;
            self.p.time += time_diff;

            self.gui.egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
            self.gui
                .egui_ctx
                .begin_frame(self.gui.egui_state.input.take());

            self.vbo.set(&self.notes.vert);
            self.ibo.set(&self.notes.ind);
            self.vao.set();
            self.p.program.set_used();
            // Try putting them in egui…….changed() {  } ?
            unsafe {
                gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
                gl::Uniform1f(self.p.u_time.id, self.p.time);
                gl::Uniform1i(self.p.u_vflip.id, self.p.vflip as i32);
                gl::Uniform3f(
                    self.p.u_ol_color.id,
                    self.p.ol_color.to_rgb()[0],
                    self.p.ol_color.to_rgb()[1],
                    self.p.ol_color.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_note_left.id,
                    self.p.note_left.to_rgb()[0],
                    self.p.note_left.to_rgb()[1],
                    self.p.note_left.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_note_right.id,
                    self.p.note_right.to_rgb()[0],
                    self.p.note_right.to_rgb()[1],
                    self.p.note_right.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_note_top.id,
                    self.p.note_top.to_rgb()[0],
                    self.p.note_top.to_rgb()[1],
                    self.p.note_top.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_note_bottom.id,
                    self.p.note_bottom.to_rgb()[0],
                    self.p.note_bottom.to_rgb()[1],
                    self.p.note_bottom.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_particle_left.id,
                    self.p.particle_left.to_rgb()[0],
                    self.p.particle_left.to_rgb()[1],
                    self.p.particle_left.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_particle_right.id,
                    self.p.particle_right.to_rgb()[0],
                    self.p.particle_right.to_rgb()[1],
                    self.p.particle_right.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_particle_top.id,
                    self.p.particle_top.to_rgb()[0],
                    self.p.particle_top.to_rgb()[1],
                    self.p.particle_top.to_rgb()[2],
                );
                gl::Uniform3f(
                    self.p.u_particle_bottom.id,
                    self.p.particle_bottom.to_rgb()[0],
                    self.p.particle_bottom.to_rgb()[1],
                    self.p.particle_bottom.to_rgb()[2],
                );
                gl::Uniform1f(
                    self.p.u_particle_transparency.id,
                    self.p.particle_transparency,
                );
            }
            self.notes.update(time_diff * self.p.gravity);
            if self.p.particle_show {
                self.particles.update(
                    time_diff * self.p.gravity,
                    &self.notes.vert,
                    self.p.particle_density as f32,
                );
            }
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::BlendEquation(gl::FUNC_ADD);
            }
            self.draw();

            if self.draw_gui() {
                break 'play;
            };
            // if draw_gui { if self.draw_gui() { break 'play }; }
            // self.draw_hider() // -> GUI that controls whether othe GUI components are visible.
            rgb = self.p.bg.to_rgb();

            let (egui_output, paint_cmds) = self.gui.egui_ctx.end_frame();
            self.gui
                .egui_state
                .process_output(&self.window, &egui_output);
            let paint_jobs = self.gui.egui_ctx.tessellate(paint_cmds);
            self.gui
                .painter
                .paint_jobs(None, paint_jobs, &self.gui.egui_ctx.font_image());

            self.window.gl_swap_window();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'play,
                    _ => {
                        // Process input event
                        self.gui.egui_state.process_input(
                            &self.window,
                            event,
                            &mut self.gui.painter,
                        );
                    }
                }
            }
        }

        self.gui
            .egui_ctx
            .begin_frame(self.gui.egui_state.input.take());
        self.vbo.set(&self.notes.vert);
        self.vao.set();
        self.ibo.set(&self.notes.ind);
        self.p.program.set_used();

        self.draw_gui();
        self.draw_last();
        unsafe {
            gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let (egui_output, paint_cmds) = self.gui.egui_ctx.end_frame();
        self.gui
            .egui_state
            .process_output(&self.window, &egui_output);
        let paint_jobs = self.gui.egui_ctx.tessellate(paint_cmds);
        self.gui
            .painter
            .paint_jobs(None, paint_jobs, &self.gui.egui_ctx.font_image());

        self.window.gl_swap_window();

        Ok(())
    }

    /// Renders the whole song to an output file, in the background.
    pub fn full_mp4(&mut self) -> Result<(), String> {
        #[cfg(debug_assertions)]
        let init_time = Instant::now();
        self.particles = Particles::new();
        self.to_start();
        self.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();

        self.vbo.set(&self.notes.vert);
        self.vao.set();
        self.ibo.set(&self.notes.ind);
        self.p.program.set_used();

        let tex = Textures::gen(); // Both standard and multisample
        tex.set(self.p.width as i32, self.p.height as i32, self.p.samples);
        let fbo = Fbos::gen(); // Both standard and multisample
        fbo.set(tex);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendEquation(gl::FUNC_ADD);
        }

        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::Viewport(0, 0, self.p.width as i32, self.p.height as i32);
        }

        unsafe {
            let rgb: [f32; 3] = self.p.bg.to_rgb();
            gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
            gl::Uniform1i(self.p.u_vflip.id, (self.p.vflip as i32 + 1) % 2);
            gl::Uniform3f(
                self.p.u_ol_color.id,
                self.p.ol_color.to_rgb()[0],
                self.p.ol_color.to_rgb()[1],
                self.p.ol_color.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_note_left.id,
                self.p.note_left.to_rgb()[0],
                self.p.note_left.to_rgb()[1],
                self.p.note_left.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_note_right.id,
                self.p.note_right.to_rgb()[0],
                self.p.note_right.to_rgb()[1],
                self.p.note_right.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_note_top.id,
                self.p.note_top.to_rgb()[0],
                self.p.note_top.to_rgb()[1],
                self.p.note_top.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_note_bottom.id,
                self.p.note_bottom.to_rgb()[0],
                self.p.note_bottom.to_rgb()[1],
                self.p.note_bottom.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_particle_left.id,
                self.p.particle_left.to_rgb()[0],
                self.p.particle_left.to_rgb()[1],
                self.p.particle_left.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_particle_right.id,
                self.p.particle_right.to_rgb()[0],
                self.p.particle_right.to_rgb()[1],
                self.p.particle_right.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_particle_top.id,
                self.p.particle_top.to_rgb()[0],
                self.p.particle_top.to_rgb()[1],
                self.p.particle_top.to_rgb()[2],
            );
            gl::Uniform3f(
                self.p.u_particle_bottom.id,
                self.p.particle_bottom.to_rgb()[0],
                self.p.particle_bottom.to_rgb()[1],
                self.p.particle_bottom.to_rgb()[2],
            );
            gl::Uniform1f(
                self.p.u_particle_transparency.id,
                self.p.particle_transparency,
            );
        }

        let mut data_ptr: [*mut u8; 2] = [null_mut(), null_mut()];

        video_builder::init().unwrap();
        let metadata = HashMap::new();
        // Those are or encoder. information to give here ?
        // metadata.insert("preset".to_owned(), "ultrafast".to_owned());
        // metadata.insert("crf".to_owned(), "0".to_owned());

        // User choice between compression and speed (and quality). (as well as HW usage)
        let options = VideoOptions {
            output_path: self.p.mkv_file.clone(),
            metadata,
            video_time_base: Rational(1, self.p.framerate),
            video_codec: "libx264rgb".to_owned(),
            video_codec_params: Default::default(),
            pixel_format_in: "rgb24".to_string(),
            pixel_format_out: "rgb24".to_string(),
            resolution_in: (self.p.width as u32, self.p.height as u32),
            resolution_out: (self.p.width as u32, self.p.height as u32),
        };
        let mut encoder = VideoBuilder::new(options, &self.p.x264_preset).unwrap();
        encoder.start_encoding().unwrap();

        let pbo = [Pbo::gen(), Pbo::gen()];

        unsafe {
            gl::Uniform1f(self.p.u_time.id, self.p.time);
        }

        #[cfg(debug_assertions)]
        let time = Instant::now();

        fbo.bind(gl::FRAMEBUFFER, fbo.m);
        self.draw();

        #[cfg(debug_assertions)]
        println!("Draw: {:?}", time.elapsed());

        #[cfg(debug_assertions)]
        let time = Instant::now();

        fbo.bind(gl::READ_FRAMEBUFFER, fbo.m);
        fbo.bind(gl::DRAW_FRAMEBUFFER, fbo.s);
        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        unsafe {
            gl::BlitFramebuffer(
                0,
                0,
                self.p.width as GLint,
                self.p.height as GLint,
                0,
                0,
                self.p.width as GLint,
                self.p.height as GLint,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }

        #[cfg(debug_assertions)]
        println!("Blit: {:?}", time.elapsed());

        #[cfg(debug_assertions)]
        let time = Instant::now();

        fbo.bind(gl::FRAMEBUFFER, fbo.s);
        pbo[0].set(self.p.bytes);
        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        self.read();

        #[cfg(debug_assertions)]
        println!("Read: {:?}", time.elapsed());

        #[cfg(debug_assertions)]
        let time = Instant::now();

        data_ptr[0] = pbo[0].map().cast();

        #[cfg(debug_assertions)]
        println!("Map ptr: {:?}", data_ptr[0]);
        #[cfg(debug_assertions)]
        println!("Map: {:?}", time.elapsed());
        #[cfg(debug_assertions)]
        let time = Instant::now();

        pbo[0].unmap();

        #[cfg(debug_assertions)]
        println!("Unmap: {:?}", time.elapsed());

        unsafe {
            gl::Uniform1f(self.p.u_time.id, self.p.time);
        }

        #[cfg(debug_assertions)]
        let time = Instant::now();

        fbo.bind(gl::FRAMEBUFFER, fbo.m);
        self.draw();

        #[cfg(debug_assertions)]
        println!("Draw: {:?}", time.elapsed());

        #[cfg(debug_assertions)]
        let time = Instant::now();

        fbo.bind(gl::READ_FRAMEBUFFER, fbo.m);
        fbo.bind(gl::DRAW_FRAMEBUFFER, fbo.s);
        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        unsafe {
            gl::BlitFramebuffer(
                0,
                0,
                self.p.width as GLint,
                self.p.height as GLint,
                0,
                0,
                self.p.width as GLint,
                self.p.height as GLint,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }

        #[cfg(debug_assertions)]
        println!("Blit: {:?}", time.elapsed());

        #[cfg(debug_assertions)]
        println!("-------------------- Frame {:06}: {:?}\n", 0, init_time.elapsed());

        'record: loop {
            let full_time = Instant::now();
            // ALSO DRAW THE NEXT FRAME WHILE MAPPING
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'record,
                    _ => {}
                }
            }

            let encodeindex = self.frame_count % 2;
            let readindex = (self.frame_count + 1) % 2;
            #[cfg(debug_assertions)]
            println!("Encode pointer: {:?}", data_ptr[encodeindex]);
            #[cfg(debug_assertions)]
            println!("Read pointer: {:?}", data_ptr[readindex]);

            #[cfg(debug_assertions)]
            let time = Instant::now();

            fbo.bind(gl::FRAMEBUFFER, fbo.s);
            pbo[readindex].set(self.p.bytes);
            unsafe {
                gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            }
            self.read();

            #[cfg(debug_assertions)]
            println!("Read: {:?}", time.elapsed());

            #[cfg(debug_assertions)]
            let time = Instant::now();

            encoder
                .push_video_data(unsafe { from_raw_parts(data_ptr[encodeindex], self.p.bytes) })
                .unwrap();
            encoder.step_encoding().unwrap();

            #[cfg(debug_assertions)]
            println!("Encode: {:?}", time.elapsed());
            #[cfg(debug_assertions)]
            let time = Instant::now();

            data_ptr[readindex] = pbo[readindex].map().cast();

            #[cfg(debug_assertions)]
            println!("Map ptr: {:?}", data_ptr[readindex]);
            #[cfg(debug_assertions)]
            println!("Map: {:?}", time.elapsed());

            #[cfg(debug_assertions)]
            let time = Instant::now();

            self.p.time += 1.0 / self.p.framerate as f32 * self.p.gravity;

            if self.p.time > self.p.max_time {
                break 'record;
            }
            unsafe {
                gl::Uniform1f(self.p.u_time.id, self.p.time);
            }

            #[cfg(debug_assertions)]
            println!("frame_count-related updates: {:?}", time.elapsed());
            #[cfg(debug_assertions)]
            let time = Instant::now();

            self.notes
                .update(1.0 / self.p.framerate as f32 * self.p.gravity);
            if self.p.particle_show {
                self.particles.update(
                    1.0 / self.p.framerate as f32 * self.p.gravity,
                    &self.notes.vert,
                    self.p.particle_density as f32,
                );
            }

            #[cfg(debug_assertions)]
            println!("Update: {:?}", time.elapsed());

            #[cfg(debug_assertions)]
            let time = Instant::now();

            fbo.bind(gl::FRAMEBUFFER, fbo.m);
            self.draw();

            #[cfg(debug_assertions)]
            println!("Draw: {:?}", time.elapsed());
            #[cfg(debug_assertions)]
            let time = Instant::now();

            fbo.bind(gl::READ_FRAMEBUFFER, fbo.m);
            fbo.bind(gl::DRAW_FRAMEBUFFER, fbo.s);
            unsafe {
                gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            }
            unsafe {
                gl::BlitFramebuffer(
                    0,
                    0,
                    self.p.width as GLint,
                    self.p.height as GLint,
                    0,
                    0,
                    self.p.width as GLint,
                    self.p.height as GLint,
                    gl::COLOR_BUFFER_BIT,
                    gl::NEAREST, // choose
                );
            }

            #[cfg(debug_assertions)]
            println!("Blit: {:?}", time.elapsed());
            #[cfg(debug_assertions)]
            let time = Instant::now();

            pbo[readindex].unmap();

            #[cfg(debug_assertions)]
            println!("Unmap: {:?}", time.elapsed());

            self.frame_count += 1;

            println!("-------------------- Frame {:06}: {:?}\n", self.frame_count, full_time.elapsed());
        }

        encoder.finish_encoding().unwrap();

        Ok(())
    }

    // /// [NOT IMPLEMENTED YET] Renders a PNG of the full song.
    // fn full_png(&mut self) -> Result<(), String> {
    //     // let ogl = self.handles.remove(0).join().unwrap();
    //
    //     self.to_start();
    //     self.particles = Particles::new();
    //     for y in self.notes.vert.iter_mut().skip(1).step_by(3) {
    //         *y = (*y / self.p.max_time as f32 - 0.5) * 2.;
    //     }
    //
    //     unsafe {
    //         gl::Uniform1f(self.p.u_time.id, 0.0);
    //     }
    //     // unsafe { gl::Viewport(0, 0, (self.width/4) as i32, (self.height*3) as i32); } // with framebuffer change as well
    //     self.draw();
    //     self.read();
    //     // let png_file = self.p.png_file.clone();
    //     // spawn(move ||{
    //     // HERE self.export_png(&png_file);
    //     #[cfg(debug_assertions)]
    //     println!("✨ Generated an image of the full song! ✨");
    //     // self.renderer.frame += self.renderer.cores;
    //     // });
    //     // self.renderer = OpenGLContext::new(self.p.width, self.p.height, self.p.framerate, self.p.cores, &self.p.midi_file);
    //
    //     // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));
    //
    //     Ok(())
    // }

    /// Draws the GUI. The return value specifes whether
    fn draw_gui(&mut self) -> bool {
        let mut start: bool = false;
        egui::Window::new("Preview")
            .default_pos((10., 10.))
            .show(&self.gui.egui_ctx, |ui| {
                egui::Grid::new("Preview").show(ui, |ui| {
                    ui.label("Preview speed:");
                    ui.add(egui::Slider::new(&mut self.p.preview_speed, -100.0..=100.0));
                    ui.end_row();

                    ui.label("Restart preview: ");
                    if ui.add(egui::Button::new("Restart")).clicked() {
                        self.notes.update(-self.p.time * self.p.gravity);
                        self.p.time = 0.;
                        self.particles = Particles::new();
                    }
                    ui.end_row();
            });
        });
        egui::Window::new("General").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("General specification").show(ui, |ui| {
                ui.label("Width:");
                if ui
                    .add(egui::Slider::new(&mut self.p.width, 1..=7680))
                    .changed()
                {
                    self.p.bytes = self.p.width * self.p.height * 3; // Multiply by 3 as this is RGB24
                }
                ui.end_row();

                ui.label("Height:");
                if ui
                    .add(egui::Slider::new(&mut self.p.height, 1..=4320))
                    .changed()
                {
                    self.p.bytes = self.p.width * self.p.height * 3; // Multiply by 3 as this is RGB24
                }
                ui.end_row();

                // ui.label("CPU Cores:");
                // ui.add(egui::Slider::new(&mut self.p.cores, 1..=self.p.max_cores));
                // ui.end_row();

                ui.label("Samples:");
                if ui
                    .add(egui::Slider::new(
                        &mut self.p.samples,
                        1..=self.p.max_samples,
                    ))
                    .changed()
                {
                    self.sdl
                        .video()
                        .unwrap()
                        .gl_attr()
                        .set_multisample_samples(self.p.samples);
                }
                ui.end_row();

                ui.label("Framerate:");
                ui.add(egui::Slider::new(&mut self.p.framerate, 1..=240));
                ui.end_row();

                ui.label("Vertical flip:");
                ui.add(egui::Checkbox::new(&mut self.p.vflip, ""));
                ui.end_row();
            });
        });
        egui::Window::new("Files").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("Files").show(ui, |ui| {
                ui.label("Input .mid file: ");
                if ui.add(egui::Button::new("Browse…")).clicked() {
                    let path = FileDialog::new()
                        .set_location(&std::env::current_dir().unwrap())
                        .add_filter("MIDI File", &["mid"])
                        .show_open_single_file()
                        .unwrap();

                    let path = match path {
                        Some(path) => path,
                        None => return,
                    };
                    println!("Opening midi file: {:?}", path);
                    
                    self.p.midi_file = path;
                    (self.notes, self.p.max_time) =
                        Notes::from_midi(800 as f32 / 600 as f32, 1.0, &self.p.midi_file).unwrap();
                    self.vbo.set(&self.notes.vert);
                    self.vao.set();
                    self.ibo.set(&self.notes.ind);
                    self.p.time = 0.;
                }
                ui.end_row();

                ui.label("Output (.mkv or .avi): ");
                ui.add(egui::TextEdit::singleline(&mut self.p.mkv_file));
                ui.end_row();
            });
        });
        egui::Window::new("Render").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("Render").show(ui, |ui| {
                ui.label("Speed vs Compression: ");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.p.x264_preset))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.p.x264_preset, "ultrafast".to_owned(), "Ultrafast");
                        ui.selectable_value(&mut self.p.x264_preset, "superfast".to_owned(), "Superfast");
                        ui.selectable_value(&mut self.p.x264_preset, "veryfast".to_owned(), "Veryfast");
                        ui.selectable_value(&mut self.p.x264_preset, "faster".to_owned(), "Faster");
                        ui.selectable_value(&mut self.p.x264_preset, "fast".to_owned(), "Fast");
                        ui.selectable_value(&mut self.p.x264_preset, "medium".to_owned(), "Medium");
                        ui.selectable_value(&mut self.p.x264_preset, "slow".to_owned(), "Slow");
                        ui.selectable_value(&mut self.p.x264_preset, "slower".to_owned(), "Slower");
                        ui.selectable_value(&mut self.p.x264_preset, "veryslow".to_owned(), "Veryslow");
                    });
                ui.end_row();

                ui.label("✨ Start rendering: ");
                if ui.add(egui::Button::new("Start")).clicked() {
                    start = true;
                }
                ui.end_row();
            });
        });
        egui::Window::new("Notes")
            .default_pos((300., 10.))
            .show(&self.gui.egui_ctx, |ui| {
                egui::Grid::new("Notes").show(ui, |ui| {
                    ui.label("Show:");
                    ui.add(egui::Checkbox::new(&mut self.p.note_show, ""));
                    ui.end_row();
                    ui.label("Gravity:");
                    if ui
                        .add(egui::Slider::new(&mut self.p.gravity, 0.3..=2.0))
                        .changed()
                    {
                        self.notes
                            .notes_to_vertices(
                                self.p.width as f32 / self.p.height as f32,
                                self.p.gravity,
                            )
                            .unwrap();
                        self.notes.update(self.p.time * self.p.gravity);
                        self.p.latest_gravity = self.p.gravity;
                    };
                    ui.end_row();

                    ui.label("Color - Left");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.note_left,
                        self.p.alpha,
                    );
                    ui.end_row();

                    ui.label("Color - Right");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.note_right,
                        self.p.alpha,
                    );
                    ui.end_row();

                    ui.label("Color - Top");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.note_top,
                        self.p.alpha,
                    );
                    ui.end_row();

                    ui.label("Color - Bottom");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.note_bottom,
                        self.p.alpha,
                    );
                    ui.end_row();
            });
        });
        egui::Window::new("Particles").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("Particles").show(ui, |ui| {
                ui.label("Show:");
                ui.add(egui::Checkbox::new(&mut self.p.particle_show, ""));
                ui.end_row();

                ui.label("Density:");
                ui.add(egui::Slider::new(&mut self.p.particle_density, 1..=50000));
                ui.end_row();

                ui.label("Transparency:");
                ui.add(egui::Slider::new(
                    &mut self.p.particle_transparency,
                    0.0..=1.0,
                ));
                ui.end_row();

                ui.label("Color - Left");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_left,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Color - Right");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_right,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Color - Top");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_top,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Color - Bottom");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_bottom,
                    self.p.alpha,
                );
                ui.end_row();
            });
        });
        egui::Window::new("Octave lines")
            .default_pos((580., 10.))
            .show(&self.gui.egui_ctx, |ui| {
                egui::Grid::new("Octave lines").show(ui, |ui| {
                    ui.label("Show:");
                    ui.add(egui::Checkbox::new(&mut self.p.ol_show, ""));
                    ui.end_row();

                    ui.label("Color");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.ol_color,
                        self.p.alpha,
                    );
                    ui.end_row();
            });
        });
        egui::Window::new("Background")
            .default_pos((580., 105.))
            .show(&self.gui.egui_ctx, |ui| {
                egui::Grid::new("Background").show(ui, |ui| {
                    ui.label("Color:");
                    egui::widgets::color_picker::color_edit_button_hsva(
                        ui,
                        &mut self.p.bg,
                        self.p.alpha,
                    );
                    ui.end_row();

                    // ui.label("Image: ");
                    // if ui.add(egui::Button::new("Find")).clicked() {
                    //     // self.notes.update(-self.p.time * self.p.gravity);
                    //     // self.p.time = 0.;
                    //     // self.frame = 0;
                    //     // self.particles = Particles::new();
                    // }
                    // ui.end_row();
            });
        });
        start
    }

    /// Draw the GUI for the last frame, which won't be updated
    fn draw_last(&mut self) {
        egui::Window::new("Pianorium").default_pos((500., 500.)).show(&self.gui.egui_ctx, |ui| {
            ui.label("Rendering started. This window will exit when the rendering finishes.");
            ui.label("Closing this window manually would stop the rendering.");
            ui.label(
                "You can minimize this window to let the rendering finish with the window hidden.",
            );
        });
    }

    // fn zero(&mut self) -> std::io::Result<()> {
    //     Ok(())
    // }

    /// Reads RGB from the bound FBO
    pub fn read(&mut self) {
        unsafe {
            gl::ReadPixels(
                0,
                0,
                self.p.width as i32,
                self.p.height as i32,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                null::<u8>() as *mut gl::types::GLvoid,
            );
        }
    }

    /// Draws the scene elements
    pub fn draw(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if self.p.ol_show {
                self.draw_ol();
            }
            if self.p.note_show {
                self.draw_notes();
            }
            if self.p.particle_show {
                self.draw_particles();
            }
        }
    }

    #[inline(always)]
    pub fn draw_ol(&mut self) {
        unsafe {
            self.vbo.set(&self.ol.vert);
            self.ibo.set(&self.ol.ind);
            gl::DrawElements(
                gl::TRIANGLES,
                self.ol.ind.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn to_start(&mut self) {
        self.notes.update(-self.p.time * self.p.gravity);
        self.p.time = 0.;
        self.frame_count = 0;
    }

    #[inline(always)]
    pub fn to_time(&mut self, time: f32) {
        self.notes.update((self.p.time - time) * self.p.gravity);
    }

    #[inline(always)]
    pub fn to_end(&mut self) {
        self.notes
            .update((self.p.max_time - self.p.time) * self.p.gravity);
        self.p.time = self.p.max_time;
        self.frame_count = (self.p.max_time * self.p.framerate as f32) as usize;
    }
}

/// Vertical Octave Lines between each B and C note
pub struct Ol {
    halfspan: f32,
    vert: Vec<f32>,
    ind: Vec<u32>,
}

impl Ol {
    pub fn create(halfspan: f32) -> std::io::Result<Self> {
        let mut ol = Ol {
            halfspan,
            vert: vec![],
            ind: vec![],
        };
        ol.ol_to_vertind().unwrap();
        Ok(ol)
    }

    /// Can be called before "notes_to_vertices" to display octave delimiters.
    fn ol_to_vertind(&mut self) -> std::io::Result<()> {
        for (i, x) in [
            -24. / 26.,
            -17. / 26.,
            -10. / 26.,
            -3. / 26.,
            4. / 26.,
            11. / 26.,
            18. / 26.,
            25. / 26.,
        ]
        .iter()
        .enumerate()
        {
            let ver2: Vec<f32> = vec![
                //        x                 y           color
                x - self.halfspan,
                -0.95,
                0.7,
                x + self.halfspan,
                -0.95,
                0.7,
                x + self.halfspan,
                0.95,
                0.7,
                x - self.halfspan,
                0.95,
                0.7,
            ];
            self.vert.extend(ver2);

            let i2: u32 = i as u32;
            let ind2: Vec<u32> = vec![
                4 * i2,
                4 * i2 + 2,
                4 * i2 + 1,
                4 * i2,
                4 * i2 + 2,
                4 * i2 + 3,
            ];
            self.ind.extend(ind2);
        }

        Ok(())
    }
}

/// Characteristics for a single note
#[derive(Debug, Clone)]
pub struct Note {
    pub note: u8, // A0 is 21 ; C8 is 108
    pub start: f32,
    pub end: f32,
}

/// Group & display of all the notes
#[derive(Clone)]
pub struct Notes {
    pub notes: Vec<Note>,
    pub vert: Vec<f32>,
    pub ind: Vec<u32>,
}

impl Notes {
    pub fn new() -> Self {
        Notes {
            notes: vec![],
            vert: vec![],
            ind: vec![],
        }
    }

    pub fn from_midi(
        wh_ratio: f32,
        gravity: f32,
        midi_file: &PathBuf,
    ) -> std::io::Result<(Notes, f32)> {
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
        print!("\nReading {}-byte midi file ", numbytes);
        let midi_data = Smf::parse(&buf).unwrap();

        let mut spb: f32 = 0.5; // Seconds per beat
        let mut spt: f32; // Seconds per tick
        match midi_data.header.timing {
            Metrical(m) => {
                let ppq: f32 = <u15 as Into<u16>>::into(m) as f32;
                spt = spb / ppq;
                #[cfg(debug_assertions)]
                println!("with Metrical timing…");
            }
            Timecode(fps, sfpf) => {
                spt = 1. / fps.as_f32() / sfpf as f32;
                #[cfg(debug_assertions)]
                println!("with Timecode timing…");
            }
        }
        let mut max_time: f32 = 0.0;

        for track in midi_data.tracks.iter() {
            let mut current_time: f32 = 2.5 / gravity;
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
                                max_time = current_time + 4. / gravity;
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

        new.notes_to_vertices(wh_ratio, gravity).unwrap();
        new.notes_to_indices().unwrap();

        Ok((new, max_time))
    }

    pub fn notes_to_vertices(&mut self, wh_ratio: f32, gravity: f32) -> std::io::Result<()> {
        self.vert = vec![];
        for n in self.notes.iter() {
            self.vert.extend(vec![
                //               x                      y            color
                LAYOUT[n.note as usize - 21][0],
                (gravity * n.start),
                1.0,
                LAYOUT[n.note as usize - 21][1],
                (gravity * n.start),
                1.0,
                LAYOUT[n.note as usize - 21][1],
                (gravity * n.end),
                1.0,
                LAYOUT[n.note as usize - 21][0],
                (gravity * n.end),
                1.0,
                //               x                                           y                    color
                LAYOUT[n.note as usize - 21][0] + 0.007,
                ((n.start + 0.007 * wh_ratio) * gravity),
                0.9,
                LAYOUT[n.note as usize - 21][1] - 0.007,
                ((n.start + 0.007 * wh_ratio) * gravity),
                0.9,
                LAYOUT[n.note as usize - 21][1] - 0.007,
                ((n.end - 0.007 * wh_ratio) * gravity),
                0.9,
                LAYOUT[n.note as usize - 21][0] + 0.007,
                ((n.end - 0.007 * wh_ratio) * gravity),
                0.9,
            ]);
        }

        Ok(())
    }

    pub fn notes_to_indices(&mut self) -> std::io::Result<()> {
        self.ind = vec![];
        for i in 0..self.notes.len() {
            self.ind.extend(vec![
                0 + 8 * i as u32,
                2 + 8 * i as u32,
                1 + 8 * i as u32,
                0 + 8 * i as u32,
                2 + 8 * i as u32,
                3 + 8 * i as u32,
                4 + 8 * i as u32,
                6 + 8 * i as u32,
                5 + 8 * i as u32,
                4 + 8 * i as u32,
                6 + 8 * i as u32,
                7 + 8 * i as u32,
            ]);
        }
        Ok(())
    }

    pub fn update(&mut self, diff: f32) {
        for y in self.vert.iter_mut().skip(1).step_by(3) {
            *y -= diff;
        }
    }
}

/// OpenGL Framebuffer Object
pub struct Fbos {
    pub s: GLuint,
    pub m: GLuint,
}

impl Drop for Fbos {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Fbos {
    pub fn gen() -> Self {
        let mut s: GLuint = 0;
        let mut m: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut s);
            gl::GenFramebuffers(1, &mut m);
        }
        Fbos { s, m }
    }

    pub fn set(&self, tex: Textures) {
        self.bind(gl::FRAMEBUFFER, self.s);
        self.tex(tex.s, gl::TEXTURE_2D);
        self.check();
        self.bind(gl::FRAMEBUFFER, self.m);
        self.tex(tex.m, gl::TEXTURE_2D_MULTISAMPLE);
        self.check();
    }

    pub fn bind(&self, target: GLenum, framebuffer: GLuint) {
        unsafe {
            gl::BindFramebuffer(target, framebuffer);
        }
    }

    fn check(&self) {
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        assert!(
            status == gl::FRAMEBUFFER_COMPLETE,
            "🛑 Framebuffer wasn't successfully bound. Error {:#?}",
            status
        )
    }

    fn tex(&self, texture: GLuint, textarget: GLenum) {
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                textarget,
                texture,
                0,
            );
        }
    }

    /// Back to default window-projected framebuffer
    fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.s);
            gl::DeleteFramebuffers(1, &self.m);
        }
    }
}

/// OpenGL Index Buffer Object
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

/// OpenGL Pixel Buffer Object, for PACK operations
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
                null(),
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

/// OpenGL Textures attached to framebuffers
pub struct Textures {
    pub s: GLuint,
    pub m: GLuint,
}

impl Drop for Textures {
    fn drop(&mut self) {
        self.unbind();
        self.delete();
    }
}

impl Textures {
    pub fn gen() -> Self {
        let mut s: GLuint = 0; // Standard
        let mut m: GLuint = 0; // Multisample
        unsafe {
            gl::GenTextures(1, &mut s);
            gl::GenTextures(1, &mut m);
        }
        Textures { s, m }
    }

    pub fn set(&self, width: i32, height: i32, samples: u8) {
        self.bind();
        self.setup(width, height, samples);
    }

    fn setup(&self, width: i32, height: i32, samples: u8) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                null(),
            );

            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                samples as GLsizei,
                gl::RGBA,
                width,
                height,
                0,
            );
        }
    }

    fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.s);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.m);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
        }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteBuffers(1, &self.s);
            gl::DeleteBuffers(1, &self.m);
        }
    }
}

/// OpenGL Vertex Array Object, for a common display of notes and particles
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

/// OpenGL Vertex Buffer Object, to which are attached: ol data | notes data | particles data
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

/// Defines the behaviour (partly random) of a single particle
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

/// Group & display of all the particles
#[derive(Clone, Debug)]
pub struct Particles {
    pub particles: Vec<Particle>,
    pub vert: Vec<f32>,
    pub ind: Vec<u32>,
}

impl Particles {
    #[inline(always)]
    pub fn new() -> Self {
        Particles {
            particles: vec![],
            vert: vec![],
            ind: vec![],
        }
    }

    pub fn update(&mut self, elapsed: f32, note_vert: &Vec<f32>, density: f32) {
        for p in self.particles.iter_mut() {
            p.update(elapsed)
        }
        self.particles.retain(|p| p.lifetime > 0.);

        // To be optimized
        let mut i: usize = 0;
        while i < note_vert.len() {
            if note_vert[i + 1] < (-1.) && note_vert[i + 7] > (-1.) {
                for _ in 0..(elapsed * density) as usize {
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

/// An OpenGL program, made of vertex and fragment shaders
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

/// And OpenGL Shader for parallel computing
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

/// Creates a shader out of a file path to one
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

/// Creates the shader program for Pianorium
pub fn create_program() -> Result<Program, &'static str> {
    let vert_shader =
        Shader::from_vert_source(&CString::new(include_str!(".vert")).unwrap()).unwrap();

    let frag_shader =
        Shader::from_frag_source(&CString::new(include_str!(".frag")).unwrap()).unwrap();

    let shader_program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    shader_program.set_used();

    Ok(shader_program)
}

/// An OpenGL uniform, used by shaders for customization
pub struct Uniform {
    pub id: GLint,
}

impl Uniform {
    pub fn new(shader: u32, name: &str) -> Result<Self, &'static str> {
        let cname: CString = CString::new(name).expect("CString::new failed");
        let location: GLint = unsafe { gl::GetUniformLocation(shader, cname.as_ptr()) };
        if location==-1 { return Err("Couldn't get uniform location"); }
        Ok(Uniform { id: location })
    }
}

/// Components necessary for drawing with egui
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

/// Holds data about the egui theme.
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

/// Modified Catppuccin color theme for egui
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

fn egui_set_theme(ctx: &egui::Context, theme: Theme) {
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
            extrusion: 20.0,
        },
        popup_shadow: epaint::Shadow {
            color: theme.base,
            ..old.popup_shadow
        },
        ..old
    });
}
