use std::{
    ffi::{c_void, CString},
    fs::File,
    io::Write,
    time::Instant,
};

use ffmpeg_sys_next::{AVCodec, AVCodecContext, AVFrame, AVPacket, AVRational, SwsContext};

pub struct Encoder<'a> {
    // codec_ptr: *mut AVCodec,
    codec: &'a mut AVCodec,

    c_ptr: *mut AVCodecContext,
    c: &'a mut AVCodecContext,

    f: File,

    pkt_ptr: *mut AVPacket,

    frame_rgb_ptr: *mut AVFrame,
    pub frame_rgb: &'a mut AVFrame,

    frame_yuv_ptr: *mut AVFrame,
    pub frame_yuv: &'a mut AVFrame,

    sws_context_ptr: *mut SwsContext,
}

impl Drop for Encoder<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::sws_freeContext(self.sws_context_ptr);
            sys::av_frame_free(self.frame_rgb_ptr.cast());
            sys::av_frame_free(self.frame_yuv_ptr.cast());
            sys::av_packet_free(self.pkt_ptr.cast());
            sys::avcodec_free_context(self.c_ptr.cast());
        }
    }
}

impl<'enc> Encoder<'enc> {
    pub fn new(width: i32, height: i32, framerate: f64, pools: u8) -> Self {
        let codec_ptr: *mut AVCodec =
            unsafe { sys::avcodec_find_encoder(sys::AVCodecID::AV_CODEC_ID_HEVC) as *mut AVCodec };
        unsafe {
            codec_ptr.as_ref().expect("\n🛑 HEVC Not Found!\n");
        }
        let codec: &mut AVCodec = unsafe { codec_ptr.as_mut() }.unwrap();

        let c_ptr: *mut AVCodecContext = unsafe { sys::avcodec_alloc_context3(codec) };
        unsafe {
            c_ptr
                .as_ref()
                .expect("\n🛑 Couldn't allocate memory for the HEVC context!\n");
        }
        let c: &mut AVCodecContext = unsafe { c_ptr.as_mut() }.unwrap();

        Self::set_opts((*c).priv_data, pools);

        let pkt_ptr: *mut AVPacket = unsafe { sys::av_packet_alloc() };

        let rat = unsafe { sys::av_d2q(framerate as f64, i32::MAX) };
        // c.rc_max_rate = 10_000_000;
        // c.bit_rate = 5_000_000;
        c.width = width as i32;
        c.height = height as i32;
        c.time_base = AVRational::from(rat);
        c.framerate = AVRational::from(AVRational {
            num: rat.den,
            den: rat.num,
        });
        c.gop_size = 10;
        c.max_b_frames = 1;
        c.pix_fmt = sys::AVPixelFormat::AV_PIX_FMT_YUV420P;
        let f =
            std::fs::File::create("output.hevc").expect("\n🛑 Couldn't create the output file!\n");

        let frame_rgb_ptr: *mut AVFrame = unsafe { sys::av_frame_alloc() };
        unsafe {
            frame_rgb_ptr
                .as_ref()
                .expect("\n🛑 Couldn't allocate memory for the RGB frame!\n");
        }
        let frame_rgb: &mut AVFrame = unsafe { frame_rgb_ptr.as_mut() }.unwrap();

        frame_rgb.format = sys::AVPixelFormat::AV_PIX_FMT_RGB24 as i32;
        frame_rgb.width = c.width;
        frame_rgb.height = c.height;
        assert!(
            unsafe { sys::av_frame_get_buffer(frame_rgb_ptr, 0) >= 0 },
            "🛑 Couldn't allocate memory for the RGB video buffer!"
        );

        let frame_yuv_ptr: *mut AVFrame = unsafe { sys::av_frame_alloc() };
        unsafe {
            frame_yuv_ptr
                .as_ref()
                .expect("🛑 Couldn't allocate memory for the YUV frame!");
        }
        let frame_yuv: &mut AVFrame = unsafe { frame_yuv_ptr.as_mut() }.unwrap();

        frame_yuv.format = sys::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
        frame_yuv.width = c.width;
        frame_yuv.height = c.height;
        assert!(
            unsafe { sys::av_frame_get_buffer(frame_yuv_ptr, 0) >= 0 },
            "🛑 Couldn't allocate memory for the YUV video buffer!"
        );

        let sws_context_ptr = unsafe {
            sys::sws_getContext(
                width,
                height,
                sys::AVPixelFormat::AV_PIX_FMT_RGB24,
                width,
                height,
                sys::AVPixelFormat::AV_PIX_FMT_YUV420P,
                0, // sys::SWS_BILINEAR as i32
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };

        let enc = Encoder {
            // codec_ptr,
            codec,

            c_ptr,
            c,

            f,

            pkt_ptr,

            frame_rgb_ptr,
            frame_rgb,

            frame_yuv_ptr,
            frame_yuv,

            sws_context_ptr,
        };
        
        assert!( unsafe { sys::avcodec_open2(c_ptr, codec_ptr, std::ptr::null_mut()) } >= 0, "\n🛑 Couldn't initialize the codec!\n");

        enc
    }

    /// Transfers the data in the RGB buffer to the YUV buffer, which is required by HEVC
    pub fn convert(&mut self, frame_count: i64, height: i32) {
        self.frame_yuv.pts = frame_count;

        unsafe {
            sys::sws_scale(
                self.sws_context_ptr,
                self.frame_rgb.data.as_mut_ptr().cast(),
                self.frame_rgb.linesize.as_mut_ptr(),
                0,
                height,
                self.frame_yuv.data.as_mut_ptr(),
                self.frame_yuv.linesize.as_mut_ptr(),
            );
        }
    }

    /// Encodes the YUV frame buffer and writes it to the output file
    pub fn encode(&mut self) {
        #[cfg(debug_assertions)]
        println!("frame_yuv_ptr: {:?}", self.frame_yuv_ptr);
        #[cfg(debug_assertions)]
        println!("frame_rgb_ptr: {:?}", self.frame_rgb_ptr);

        let pkt: &mut AVPacket = unsafe { self.pkt_ptr.as_mut() }.unwrap();

        assert!(unsafe { sys::avcodec_send_frame(self.c_ptr, self.frame_yuv_ptr) } >= 0);

        let mut ret = 0i32;
        while ret >= 0 {
            ret = unsafe { sys::avcodec_receive_packet(self.c_ptr, self.pkt_ptr) };
            if ret == sys::AVERROR(sys::EAGAIN) || ret == sys::AVERROR_EOF {
                return;
            } else if ret < 0 {
                panic!("🛑 Error during encoding");
            }

            #[cfg(debug_assertions)]
            println!("Write packet no. {} of size {}", pkt.pts, pkt.size);
            self.f
                .write(unsafe { std::slice::from_raw_parts(pkt.data, pkt.size as usize) })
                .unwrap();
            unsafe { sys::av_packet_unref(self.pkt_ptr) };
        }
    }

    /// Same as `encode()`: 1. for the last frame, 2. writes EOF, and 3. frees memory
    pub fn encode_last(&mut self) {
        #[cfg(debug_assertions)]
        let time = Instant::now();

        let pkt: &mut AVPacket = unsafe { self.pkt_ptr.as_mut() }.unwrap();
        assert!(unsafe { sys::avcodec_send_frame(self.c_ptr, self.frame_yuv_ptr) } >= 0);

        let mut ret = 0i32;
        while ret >= 0 {
            ret = unsafe { sys::avcodec_receive_packet(self.c_ptr, self.pkt_ptr) };
            if ret == sys::AVERROR(sys::EAGAIN) || ret == sys::AVERROR_EOF {
                return;
            } else if ret < 0 {
                panic!("🛑 Error during encoding");
            }

            #[cfg(debug_assertions)]
            println!("Write packet no. {} of size {}", pkt.pts, pkt.size);
            self.f
                .write(unsafe { std::slice::from_raw_parts(pkt.data, pkt.size as usize) })
                .unwrap();
            unsafe { sys::av_packet_unref(self.pkt_ptr) };
        }

        let pkt: &mut AVPacket = unsafe { self.pkt_ptr.as_mut() }.unwrap();
        assert!(unsafe { sys::avcodec_send_frame(self.c_ptr, std::ptr::null_mut()) } >= 0);

        let mut ret = 0i32;
        while ret >= 0 {
            ret = unsafe { sys::avcodec_receive_packet(self.c_ptr, self.pkt_ptr) };
            if ret == sys::AVERROR(sys::EAGAIN) || ret == sys::AVERROR_EOF {
                return;
            } else if ret < 0 {
                panic!("🛑 Error during encoding");
            }

            #[cfg(debug_assertions)]
            println!("Write packet no. {} of size {}", pkt.pts, pkt.size);
            self.f
                .write(unsafe { std::slice::from_raw_parts(pkt.data, pkt.size as usize) })
                .unwrap();
            unsafe { sys::av_packet_unref(self.pkt_ptr) };
        }

        // Write trailer
        if self.codec.id == sys::AVCodecID::AV_CODEC_ID_MPEG1VIDEO
            || self.codec.id == sys::AVCodecID::AV_CODEC_ID_MPEG2VIDEO
        {
            self.f.write(&[0, 0, 1, 0xb7]).unwrap();
        }

        #[cfg(debug_assertions)]
        println!("Encoded the last frame in: {:?}", time.elapsed());

        // Wait for writing streams to finish
        self.f.sync_all().unwrap();

        #[cfg(debug_assertions)]
        println!("Finished transferring to the output file");

        unsafe {
            sys::sws_freeContext(self.sws_context_ptr);
            sys::av_frame_free(self.frame_rgb_ptr.cast());
            sys::av_frame_free(self.frame_yuv_ptr.cast());
            sys::av_packet_free(self.pkt_ptr.cast());
            sys::avcodec_free_context(self.c_ptr.cast());
        }
    }

    /// Gets the number of bytes of a buffer that would have `pix_fmt` as its pixel format.
    pub fn get_size(&self, pix_fmt: sys::AVPixelFormat) -> i32 {
        unsafe { sys::av_image_get_buffer_size(pix_fmt, self.c.width, self.c.height, 0) }
    }

    fn set_opts(priv_data: *mut c_void, pools: u8) {
        let key = CString::new("preset").expect("CString conversion failed");
        let value = CString::new("veryslow").expect("CString conversion failed");
        unsafe {
            sys::av_opt_set(priv_data, key.as_ptr(), value.as_ptr(), 0);
        }

        let key = CString::new("crf").expect("CString conversion failed");
        let value = CString::new("0").expect("CString conversion failed");
        unsafe {
            sys::av_opt_set(priv_data, key.as_ptr(), value.as_ptr(), 0);
        }

        //     let key = CString::new("pools").expect("CString conversion failed");
        //     let value = CString::new(pools.to_string()).expect("CString conversion failed");
        //     unsafe {
        //         sys::av_opt_set(priv_data,
        //             key.as_ptr(),
        //             value.as_ptr(),
        //             0
        //         );
        //     }
    }
}
