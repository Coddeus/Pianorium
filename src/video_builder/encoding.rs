use std::time::Duration;
use ffmpeg_next::{Dictionary, frame, Packet};
use crate::video_builder::ffmpeg_hacks::ffmpeg_context_bytes_written;
use super::vb_unwrap::VideoBuilderUnwrap;
use super::VideoBuilder;

impl VideoBuilder {
    pub fn push_video_data(&mut self, video: &[u8]) -> Result<(), String> {
        let mut input_frame = frame::Video::new(self.v_swc_ctx.input().format, self.v_swc_ctx.input().width, self.v_swc_ctx.input().height);
        input_frame.data_mut(0).copy_from_slice(video);


        self.v_frame_buf.push_back(input_frame);

        Ok(())
    }

    fn send_video_to_encoder(&mut self) -> Result<(), String> {
        if let Some(mut frame) = self.v_frame_buf.pop_front() {
            frame.set_pts(Some(self.v_pts));
            self.v_encoder.send_frame(&frame).vb_unwrap()?;

            self.v_pts += 1;
        }

        Ok(())
    }

    fn mux_video_frame(&mut self, packet: &mut Packet) -> Result<bool, String> {
        if self.v_encoder.receive_packet(packet).is_ok() {
            let out_time_base = self.out_ctx.stream(self.v_stream_idx)
                .unwrap()
                .time_base();

            packet.rescale_ts(self.options.video_time_base, out_time_base);
            packet.set_stream(self.v_stream_idx);
            packet.write_interleaved(&mut self.out_ctx).vb_unwrap()?;

            self.v_pts_muxed += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn start_encoding(&mut self) -> Result<(), String> {
        let mut opts = Dictionary::new();
        #[cfg(debug_assertions)]
        println!("Output format: {}\n", self.out_ctx.format().name());
        match self.out_ctx.format().name() {
            "mp4" => opts.set("movflags", "faststart"),
            _ => ()
        };

        self.out_ctx.write_header_with(opts).vb_unwrap()?;

        Ok(())
    }

    pub fn step_encoding(&mut self) -> Result<(), String> {
        let mut packet = Packet::empty();

        loop {
            if !self.v_frame_buf.is_empty() {
                self.send_video_to_encoder()?;
                if !(self.mux_video_frame(&mut packet)?) {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn finish_encoding(&mut self) -> Result<(), String> {
        self.v_encoder.send_eof().vb_unwrap()?;

        let mut packet = Packet::empty();
        loop {
            let muxed_video = self.mux_video_frame(&mut packet)?;

            if !muxed_video {
                break;
            }
        }

        self.out_ctx.write_trailer().vb_unwrap()?;

        Ok(())
    }

    pub fn encoded_video_duration(&self) -> Duration {
        let time_base_fraction = self.options.video_time_base.numerator() as f64 / self.options.video_time_base.denominator() as f64;
        let seconds = time_base_fraction * self.v_pts as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn encoded_video_size(&self) -> usize {
        ffmpeg_context_bytes_written(&self.out_ctx)
    }
}
