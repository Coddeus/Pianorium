extern crate egui_sdl2_gl;
extern crate ffmpeg_next as ffmpeg;
extern crate midly;
extern crate num_cpus;
extern crate rand;

mod parameters;
use parameters::Parameters;
mod layout;
use layout::{BLACK, LAYOUT};

fn main() {
    let mut p = Pianorium::new().unwrap();
    p.play().unwrap();
    p.full_mp4().unwrap();
    p.full_png().unwrap();
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

use std::collections::HashMap;
use std::env;

use ffmpeg::{
    codec, decoder, encoder, format, frame, log, media, picture, threading::Config, Dictionary,
    Packet, Rational,
};

const DEFAULT_X264_OPTS: &str = "preset=medium";

pub struct Encoder {
    // ost_index: usize,
    // decoder: decoder::Video,
    // encoder: encoder::video::Video,
    // logging_enabled: bool,
    // frame_count: usize,
    // last_log_frame_count: usize,
    // starting_time: Instant,
    // last_log_time: Instant,
}

impl Encoder {
    // fn new(
    //     ist: &format::stream::Stream,
    //     octx: &mut format::context::Output,
    //     ost_index: usize,
    //     x264_opts: Dictionary,
    //     enable_logging: bool,
    // ) -> Result<Self, ffmpeg::Error> {
    //     let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);
    //     let decoder = ffmpeg::codec::context::Context::from_parameters(ist.parameters())?
    //         .decoder()
    //         .video()?;
    //     let mut ost = octx.add_stream(encoder::find(codec::Id::H264))?;
    //     let mut encoder = codec::context::Context::from_parameters(ost.parameters())?
    //         .encoder()
    //         .video()?;
    //     encoder.set_height(decoder.height());
    //     encoder.set_width(decoder.width());
    //     encoder.set_aspect_ratio(decoder.aspect_ratio());
    //     encoder.set_format(decoder.format());
    //     encoder.set_frame_rate(decoder.frame_rate());
    //     encoder.set_time_base(decoder.frame_rate().unwrap().invert());
    //     if global_header {
    //         encoder.set_flags(codec::Flags::GLOBAL_HEADER);
    //     }
    //
    //     encoder
    //         .open_with(x264_opts)
    //         .expect("error opening libx264 encoder with supplied settings");
    //     encoder = codec::context::Context::from_parameters(ost.parameters())?
    //         .encoder()
    //         .video()?;
    //     ost.set_parameters(&encoder);
    //     Ok(Self {
    //         ost_index,
    //         decoder,
    //         encoder: codec::context::Context::from_parameters(ost.parameters())?
    //             .encoder()
    //             .video()?,
    //         logging_enabled: enable_logging,
    //         frame_count: 0,
    //         last_log_frame_count: 0,
    //         starting_time: Instant::now(),
    //         last_log_time: Instant::now(),
    //     })
    // }
    //
    // fn send_packet_to_decoder(&mut self, packet: &Packet) {
    //     self.decoder.send_packet(packet).unwrap();
    // }
    //
    // fn send_eof_to_decoder(&mut self) {
    //     self.decoder.send_eof().unwrap();
    // }
    //
    // fn receive_and_process_decoded_frames(
    //     &mut self,
    //     octx: &mut format::context::Output,
    //     ost_time_base: Rational,
    // ) {
    //     let mut frame = frame::Video::empty();
    //     while self.decoder.receive_frame(&mut frame).is_ok() {
    //         self.frame_count += 1;
    //         let timestamp = frame.timestamp();
    //         self.log_progress(f64::from(
    //             Rational(timestamp.unwrap_or(0) as i32, 1) * self.decoder.time_base(),
    //         ));
    //         frame.set_pts(timestamp);
    //         frame.set_kind(picture::Type::None);
    //         self.send_frame_to_encoder(&frame);
    //         self.receive_and_process_encoded_packets(octx, ost_time_base);
    //     }
    // }
    //
    // fn send_frame_to_encoder(&mut self, frame: &frame::Video) {
    //     self.encoder.send_frame(frame).unwrap();
    // }
    //
    // fn send_eof_to_encoder(&mut self) {
    //     self.encoder.send_eof().unwrap();
    // }
    //
    // fn receive_and_process_encoded_packets(
    //     &mut self,
    //     octx: &mut format::context::Output,
    //     ost_time_base: Rational,
    // ) {
    //     let mut encoded = Packet::empty();
    //     while self.encoder.receive_packet(&mut encoded).is_ok() {
    //         encoded.set_stream(self.ost_index);
    //         encoded.rescale_ts(self.decoder.time_base(), ost_time_base);
    //         encoded.write_interleaved(octx).unwrap();
    //     }
    // }
    //
    // fn log_progress(&mut self, timestamp: f64) {
    //     if !self.logging_enabled
    //         || (self.frame_count - self.last_log_frame_count < 100
    //             && self.last_log_time.elapsed().as_secs_f64() < 1.0)
    //     {
    //         return;
    //     }
    //     eprintln!(
    //         "time elpased: \t{:8.2}\tframe count: {:8}\ttimestamp: {:8.2}",
    //         self.starting_time.elapsed().as_secs_f64(),
    //         self.frame_count,
    //         timestamp
    //     );
    //     self.last_log_frame_count = self.frame_count;
    //     self.last_log_time = Instant::now();
    // }
}

fn parse_opts<'a>(s: String) -> Option<Dictionary<'a>> {
    let mut dict = Dictionary::new();
    for keyval in s.split_terminator(',') {
        let tokens: Vec<&str> = keyval.split('=').collect();
        match tokens[..] {
            [key, val] => dict.set(key, val),
            _ => return None,
        }
    }
    Some(dict)
}

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

    pub frame: usize,
    pub data: Vec<u8>,
    pub ol: Ol,
    pub notes: Notes,
    pub particles: Particles,
    pub vbo: Vbo,
    pub vao: Vao,
    pub ibo: Ibo,

    // pub frames: Vec<Vec<u8>>,
    pub encoder: Encoder,
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
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(6);
        let window = video_subsystem
            .window("Pianorium", 800, 600)
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
            gl::Enable(gl::BLEND);
        }
        let event_pump: sdl2::EventPump = sdl.event_pump().unwrap();

        let mut p: Parameters = Parameters::default();

        let time = Instant::now();
        let data: Vec<u8> = vec![0; 800 * 600 * 4];
        let frame: usize = 0;
        let ol: Ol = Ol::create(p.octave_line).unwrap();

        let (notes, max_time) =
            Notes::from_midi(800 as f32 / 600 as f32, 60.0, 1.0, 0.0001, "test.mid").unwrap();
        let particles: Particles = Particles::new();
        let vbo: Vbo = Vbo::gen();
        vbo.set(&notes.vert);
        let vao: Vao = Vao::gen();
        vao.set();
        let ibo: Ibo = Ibo::gen();
        ibo.set(&notes.ind);
        unsafe {
            gl::Viewport(0, 0, 800 as i32, 600 as i32);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        }
        println!("Create OpenGLContext: {:?}", time.elapsed());

        p.max_time = max_time;

        let gui: Gui = Gui::new(&window).unwrap();
        // HANDLES FOR OPENGL
        // let handles: Vec<JoinHandle<OpenGLContext>> = fill_handles(p.width, p.height, p.framerate, &p.midi_file).unwrap();
        let encoder: Encoder = Encoder {};

        Self::setup().unwrap();

        Ok(Pianorium {
            p,
            sdl,
            window,
            gl_context,
            gl,
            event_pump,
            gui,
            frame,
            data,
            ol,
            notes,
            particles,
            vbo,
            vao,
            ibo,
            encoder,
        })
    }

    /// Plays the song with realtime changes from the GUI.
    pub fn play(&mut self) -> Result<(), String> {
        self.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();

        // let ogl = self.handles.remove(0).join().unwrap();
        // self.to_zero();
        let mut rgb: [f32; 3] = [0.1, 0.1, 0.1];

        println!("Playing the visualization…");
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
            self.vao.set();
            self.ibo.set(&self.notes.ind);
            self.p.program.set_used();
            unsafe {
                gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
                gl::Uniform1f(self.p.u_time.id, self.p.time);
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
                    self.p.u_note_time.id,
                    self.p.note_time.to_rgb()[0],
                    self.p.note_time.to_rgb()[1],
                    self.p.note_time.to_rgb()[2],
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
                gl::Uniform3f(
                    self.p.u_particle_time.id,
                    self.p.particle_time.to_rgb()[0],
                    self.p.particle_time.to_rgb()[1],
                    self.p.particle_time.to_rgb()[2],
                );
            }
            self.notes.update(time_diff * self.p.gravity);
            self.particles
                .update(time_diff * self.p.gravity, &self.notes.vert);
            self.draw();

            self.draw_gui();
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

            // if !egui_output.needs_repaint {
            //     if let Some(event) = self.event_pump.wait_event_timeout(5) {
            //         match event {
            //             Event::Quit { .. } => break 'play,
            //             _ => {
            //                 // Process input event
            //                 self.gui.egui_state.process_input(&self.window, event, &mut self.gui.painter);
            //             }
            //         }
            //     }
            // } else {

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

            // }
        }
        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        self.gui
            .egui_ctx
            .begin_frame(self.gui.egui_state.input.take());
        self.vbo.set(&self.notes.vert);
        self.vao.set();
        self.ibo.set(&self.notes.ind);
        self.p.program.set_used();

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

    /// Records the whole song in the background.
    pub fn full_mp4(&mut self) -> Result<(), String> {
        self.particles = Particles::new();
        self.to_start();
        self.window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();

        // let encoder: codec::encoder::Encoder = codec::encoder::new();
        // encoder.set_bit_rate(1_000_000);
        // encoder.set_max_bit_rate(1_500_000);
        // encoder.set_frame_rate(60.0.into());
        // encoder.set_compression(None);
        // encoder.set_threading(None);

        let mut index = File::create(self.p.index_file.clone()).unwrap();
        println!("Rendering frames…");

        self.vbo.set(&self.notes.vert);
        self.vao.set();
        self.ibo.set(&self.notes.ind);
        self.p.program.set_used();
        self.refresh();

        let tex = Texture::gen();
        tex.set(self.p.width as i32, self.p.height as i32);
        let fbo = Fbo::gen();
        fbo.set(tex.id);

        let pbo = Pbo::gen();
        pbo.set(self.p.bytes);

        unsafe {
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::Viewport(0, 0, self.p.width as i32, self.p.height as i32);
        }

        unsafe {
            let rgb: [f32; 3] = self.p.bg.to_rgb();
            gl::ClearColor(rgb[0], rgb[1], rgb[2], 1.0);
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
                self.p.u_note_time.id,
                self.p.note_time.to_rgb()[0],
                self.p.note_time.to_rgb()[1],
                self.p.note_time.to_rgb()[2],
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
            gl::Uniform3f(
                self.p.u_particle_time.id,
                self.p.particle_time.to_rgb()[0],
                self.p.particle_time.to_rgb()[1],
                self.p.particle_time.to_rgb()[2],
            );
        }

        'record: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'record,
                    _ => {} // egui_state.process_input(&window, event, &mut painter);
                }
            }
            self.p.time += 1.0 / self.p.framerate * self.p.gravity;

            // for _u in 0..self.p.cores {
            // let mut ogl = self.handles.remove(0).join().unwrap();
            if self.p.time > self.p.max_time {
                break 'record;
            } // Stop when it's finished playing
            unsafe {
                gl::Uniform1f(self.p.u_time.id, self.p.time);
            }

            let time = Instant::now();
            self.notes.update(1.0 / self.p.framerate * self.p.gravity);
            self.particles
                .update(1.0 / self.p.framerate * self.p.gravity, &self.notes.vert);
            println!("Update: {:?}", time.elapsed());
            let time = Instant::now();
            self.draw();
            println!("Draw: {:?}", time.elapsed());

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

            self.frame += 1;
            let name: String = format!("pianorium_temp/{:010}.mp4", self.frame);
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

        self.to_start();
        self.particles = Particles::new();
        for y in self.notes.vert.iter_mut().skip(1).step_by(3) {
            *y = (*y / self.p.max_time as f32 - 0.5) * 2.;
        }

        unsafe {
            gl::Uniform1f(self.p.u_time.id, 0.0);
        }
        // unsafe { gl::Viewport(0, 0, (self.width/4) as i32, (self.height*3) as i32); } // with framebuffer change as well
        self.draw();
        self.read();
        let png_file = self.p.png_file.clone();
        // spawn(move ||{
        self.export_png(&png_file);
        println!("✨ Generated an image of the full song! ✨");
        // self.renderer.frame += self.renderer.cores;
        // });
        // self.renderer = OpenGLContext::new(self.p.width, self.p.height, self.p.framerate, self.p.cores, &self.p.midi_file);

        // self.handles.insert(0, std::thread::spawn(move ||{ ogl }));

        Ok(())
    }
    /// Draws the GUI.
    fn draw_gui(&mut self) {
        egui::Window::new("Preview").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("Preview").show(ui, |ui| {
                ui.label("Preview speed:");
                ui.add(egui::Slider::new(&mut self.p.preview_speed, -100.0..=100.0));
                ui.end_row();

                ui.label("Restart preview: ");
                if ui.add(egui::Button::new("      ")).clicked() {
                    self.notes.update(-self.p.time * self.p.gravity);
                    self.p.time = 0.;
                    self.frame = 0;
                    self.particles = Particles::new();
                }
                ui.end_row();
            });
        });
        egui::Window::new("General").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("General").show(ui, |ui| {
                ui.label("Width:");
                ui.add(egui::Slider::new(&mut self.p.width, 1..=7680));
                ui.end_row();

                ui.label("Height:");
                ui.add(egui::Slider::new(&mut self.p.height, 1..=4320));
                ui.end_row();

                ui.label("CPU Cores:");
                ui.add(egui::Slider::new(&mut self.p.cores, 1..=self.p.max_cores));
                ui.end_row();

                ui.label("Samples:");
                ui.add(egui::Slider::new(&mut self.p.samples, 1..=50));
                ui.end_row();

                ui.label("Framerate:");
                ui.add(egui::Slider::new(&mut self.p.framerate, 0.0..=240.0));
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
            });
        });
        egui::Window::new("Coloring").show(&self.gui.egui_ctx, |ui| {
            egui::Grid::new("Coloring").show(ui, |ui| {
                ui.label("Background color:");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.bg,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Notes color - Left");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.note_left,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Notes color - Right");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.note_right,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Notes color - Top");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.note_top,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Notes color - Bottom");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.note_bottom,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Notes color - Time");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.note_time,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Particles color - Left");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_left,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Particles color - Right");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_right,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Particles color - Top");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_top,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Particles color - Bottom");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_bottom,
                    self.p.alpha,
                );
                ui.end_row();

                ui.label("Particles color - Time");
                egui::widgets::color_picker::color_edit_button_hsva(
                    ui,
                    &mut self.p.particle_time,
                    self.p.alpha,
                );
                ui.end_row();
            });
        });
    }

    /// Draw the sceneed while rendering
    fn draw_last(&mut self) {
        egui::Window::new("Pianorium").show(&self.gui.egui_ctx, |ui| {
            ui.label("Rendering started. This window will exit when the rendering finishes.");
            ui.label(
                "You can minimize this window to let the rendering finish with the window hidden",
            );
            ui.label("A progress bar may be added in the future.");
        });
    }

    pub fn refresh(&mut self) {
        self.p.bytes = self.p.width * self.p.height * 4;
        if self.p.samples > 1 {
            self.sdl
                .video()
                .unwrap()
                .gl_attr()
                .set_multisample_samples(self.p.samples)
        }
    }

    fn zero(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn setup() -> std::io::Result<()> {
        let _ = remove_dir_all("pianorium_temp");
        create_dir("pianorium_temp")?;
        Ok(())
    }

    fn teardown() -> std::io::Result<()> {
        remove_dir_all("pianorium_temp")?;
        let _ = remove_file("pianorium_index.txt");
        let _ = remove_file("pianorium_ff_concat_mp4.log");
        let _ = remove_file("pianorium_ff_export_mp4.log");
        let _ = remove_file("pianorium_ff_export_png.log");
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
        let name = format!("pianorium_temp/{:010}.mp4", self.frame);
        let filename = name.as_str();

        let mut ffmpeg = Command::new("ffmpeg")
            .env("FFREPORT", "file=pianorium_ff_export_mp4.log:level=56")
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
            .env("FFREPORT", "file=pianorium_ff_export_png.log:level=56")
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
            stdin.write_all(&self.data).unwrap();
        }
    }

    pub fn concat_mp4(output: &str) {
        println!("Concatenating into one video…");

        Command::new("ffmpeg")
            .env("FFREPORT", "file=pianorium_ff_concat_mp4.log:level=56")
            .arg("-loglevel")
            .arg("0")
            .arg("-f")
            .arg("concat")
            .arg("-i")
            .arg("pianorium_index.txt")
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

    pub fn draw(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            self.draw_ol();
            self.draw_notes();
            self.draw_particles();
        }
    }

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

    #[inline(always)]
    pub fn to_start(&mut self) {
        self.notes.update(-self.p.time * self.p.gravity);
        self.p.time = 0.;
        self.frame = 0;
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
        self.frame = (self.p.max_time * self.p.framerate) as usize;
    }
}

// Simulated function to generate raw RGB frame data
fn generate_raw_rgb_frame() -> Result<Vec<u8>, std::io::Error> {
    // Replace this with your real-time frame generation logic.
    // For simplicity, this function returns a green frame.
    let width = 1280;
    let height = 720;
    let bytes_per_pixel = 3; // RGB24 format (3 bytes per pixel)
    let frame_size = width * height * bytes_per_pixel;
    let mut frame_data = vec![0u8; frame_size];

    for i in 0..frame_size {
        frame_data[i] = if (i / bytes_per_pixel) % width < width / 2 {
            0 // Red
        } else {
            255 // Green
        };
    }

    Ok(frame_data)
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

// use std::sync::{Arc, Mutex};

pub struct OpenGLContext {}

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
//             gravity: self.gravity,
//             max_time: self.max_time,
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
//             gravity: self.gravity,
//             max_time: self.max_time,
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
    // /// Returns a ready-to-use context and the final frame number
    // pub fn new(
    //     width: usize,
    //     height: usize,
    //     framerate: f32,
    //     gravity: f32,
    //     octave_line: f32,
    //     midi_file: &str,
    // ) -> (Self, f32) { }

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
}

/// Vertical Octave Lines
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
        gravity: f32,
        octave_line: f32,
        midi_file: &str,
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
        print!("Reading {}-byte midi file ", numbytes);
        let midi_data = Smf::parse(&buf).unwrap();

        let mut spb: f32 = 0.5; // Seconds per beat
        let mut spt: f32; // Seconds per tick
        match midi_data.header.timing {
            Metrical(m) => {
                let ppq: f32 = <u15 as Into<u16>>::into(m) as f32;
                spt = spb / ppq;
                println!("with Metrical timing…");
            }
            Timecode(fps, sfpf) => {
                spt = 1. / fps.as_f32() / sfpf as f32;
                println!("with Timecode timing…");
            }
        }
        let mut max_time: f32 = 0.0;

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
                                max_time = current_time + 2.;
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
        self.check();
    }

    fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
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
    #[inline(always)]
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

// pub struct Winsdl {
// }
//
// impl Winsdl {
//     pub fn new(width: usize, height: usize, samples: u8) -> Result<Self, &'static str> {
//
//         Ok(Winsdl {
//             sdl,
//             window,
//             gl_context,
//             gl,
//             event_pump,
//         })
//     }
// }
