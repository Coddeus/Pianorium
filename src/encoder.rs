use std::{fs::File, io::Write, ffi::{CString, c_void}, time::Instant};

use ffmpeg_sys_next::{AVCodec, AVCodecContext, AVPacket, AVFrame, AVRational};

pub struct Encoder <'a>{
    // codec_ptr: *mut AVCodec,
    codec: &'a mut AVCodec,

    c_ptr: *mut AVCodecContext,
    c: &'a mut AVCodecContext,

    f: File,

    pkt_ptr: *mut AVPacket,

    frame_rgb_ptr: *mut AVFrame,
    pub frame_rgb: &'a mut AVFrame,

    frame_yuv_ptr: *mut AVFrame,
    frame_yuv: &'a mut AVFrame,
}

impl Drop for Encoder<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::avcodec_free_context(self.c_ptr.cast());
            sys::av_packet_free(self.pkt_ptr.cast());
            sys::av_frame_free(self.frame_rgb_ptr.cast());
            sys::av_frame_free(self.frame_yuv_ptr.cast());
        }
    }
}

impl<'enc> Encoder<'enc> {
    pub fn new(width: i32, height: i32, framerate: f64) -> Self {
        let codec_ptr: *mut AVCodec = unsafe { sys::avcodec_find_encoder(sys::AVCodecID::AV_CODEC_ID_HEVC) as *mut AVCodec };
        unsafe { codec_ptr.as_ref().expect("\n🛑 HEVC Not Found!\n"); }
        let codec: &mut AVCodec = unsafe { codec_ptr.as_mut() }.unwrap();


        let c_ptr: *mut AVCodecContext = unsafe { sys::avcodec_alloc_context3(codec) };
        unsafe { c_ptr.as_ref().expect("\n🛑 Couldn't allocate memory for the HEVC context!\n"); }
        let c: &mut AVCodecContext = unsafe { c_ptr.as_mut() }.unwrap();


        Self::set_opts((*c).priv_data);


        let pkt_ptr: *mut AVPacket = unsafe { sys::av_packet_alloc() };


        let rat = unsafe { sys::av_d2q(framerate as f64, i32::MAX) };
        c.bit_rate = 1_000_000;
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


        let f = std::fs::File::create("output.hevc").expect("\n🛑 Couldn't create the output file!\n");


        let frame_rgb_ptr: *mut AVFrame = unsafe { sys::av_frame_alloc() };
        unsafe { frame_rgb_ptr.as_ref().expect("\n🛑 Couldn't allocate memory for the RGB frame!\n"); }
        let frame_rgb: &mut AVFrame = unsafe { frame_rgb_ptr.as_mut() }.unwrap();

        frame_rgb.format = sys::AVPixelFormat::AV_PIX_FMT_RGB24 as i32;
        frame_rgb.width = c.width;
        frame_rgb.height = c.height;
        assert!(unsafe { sys::av_frame_get_buffer(frame_rgb_ptr, 0) >= 0 }, "🛑 Couldn't allocate memory for the RGB video buffer!");

        
        let frame_yuv_ptr: *mut AVFrame = unsafe { sys::av_frame_alloc() };
        unsafe { frame_yuv_ptr.as_ref().expect("🛑 Couldn't allocate memory for the YUV frame!"); }
        let frame_yuv: &mut AVFrame = unsafe { frame_yuv_ptr.as_mut() }.unwrap();
        
        frame_yuv.format = sys::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
        frame_yuv.width = c.width;
        frame_yuv.height = c.height;
        assert!(unsafe { sys::av_frame_get_buffer(frame_yuv_ptr, 0) >= 0 }, "🛑 Couldn't allocate memory for the YUV video buffer!");
        

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
        };

        let sizergb = enc.get_size(sys::AVPixelFormat::AV_PIX_FMT_RGB24);
        let sizeyuv = enc.get_size(sys::AVPixelFormat::AV_PIX_FMT_YUV420P);

        #[cfg(debug_assertions)]
        println!("Color buffers: \nRGB size: {}\nYUV size: {}\n", sizergb, sizeyuv);

        assert!(sizergb >= 0 , "\n🛑 The size of the image buffer is negative ?!\n");
        assert!(sizeyuv >= 0 , "\n🛑 The size of the image buffer is negative ?!\n");
        assert!(unsafe { sys::avcodec_open2(c_ptr, codec_ptr, std::ptr::null_mut()) } >= 0 , "\n🛑 Couldn't initialize the codec!\n");


        enc
    }

    pub fn encode(&mut self) {
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
            self.f.write(unsafe { std::slice::from_raw_parts(pkt.data, pkt.size as usize) }).unwrap();
            unsafe { sys::av_packet_unref(self.pkt_ptr) };
        }
    }
    
    pub fn encode_last(&mut self) {
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
            self.f.write(unsafe { std::slice::from_raw_parts(pkt.data, pkt.size as usize) }).unwrap();
            unsafe { sys::av_packet_unref(self.pkt_ptr) };
        }
        
        // Write trailer
        if self.codec.id == sys::AVCodecID::AV_CODEC_ID_MPEG1VIDEO
            || self.codec.id == sys::AVCodecID::AV_CODEC_ID_MPEG2VIDEO
        {
            self.f.write(&[0, 0, 1, 0xb7]).unwrap();
        }

        self.f.sync_all().unwrap();
    }

    pub fn send(&mut self, frame: i64, ptr: *mut u8) {
        #[cfg(debug_assertions)]
        println!("frame_yuv_ptr: {:?}", self.frame_yuv_ptr);
        println!("frame_rgb_ptr: {:?}", self.frame_rgb_ptr);

        let ret = unsafe { sys::av_frame_make_writable(self.frame_rgb_ptr) };
        if ret < 0 {
            let mut buf = [0u8; 256];
            unsafe {
                sys::av_strerror(ret, buf.as_mut_ptr() as *mut i8, buf.len());
            }
            panic!("🛑 Error during av_frame_make_writable: {}", String::from_utf8_lossy(&buf));
        }

        #[cfg(debug_assertions)]
        println!("av_frame_make_writable: {}", unsafe {sys::av_frame_make_writable(self.frame_rgb_ptr)});
        assert!(unsafe { sys::av_frame_make_writable(self.frame_rgb_ptr) } >= 0);
        // self.frame_rgb.data[0] = ptr; ⚠ Leaks mem \/ Manual AVBufferRef ?

        // SwScale here
        
        let ret = unsafe { sys::av_frame_make_writable(self.frame_yuv_ptr) };
        if ret < 0 {
            let mut buf = [0u8; 256];
            unsafe {
                sys::av_strerror(ret, buf.as_mut_ptr() as *mut i8, buf.len());
            }
            panic!("🛑 Error during av_frame_make_writable: {}", String::from_utf8_lossy(&buf));
        }

        #[cfg(debug_assertions)]
        println!("av_frame_make_writable: {}", unsafe {sys::av_frame_make_writable(self.frame_yuv_ptr)});
        assert!(unsafe { sys::av_frame_make_writable(self.frame_yuv_ptr) } >= 0);
        self.frame_yuv.pts = frame;

        #[cfg(debug_assertions)]
        let time = Instant::now();
        
        self.encode();

        #[cfg(debug_assertions)]
        println!("Encode: {}", time.elapsed().as_secs_f32());
    }

    pub fn get_size(&self, pix_fmt: sys::AVPixelFormat) -> i32 {
        unsafe { sys::av_image_get_buffer_size(
            pix_fmt,
            self.c.width,
            self.c.height,
            0,
        )}
    }
    
    fn set_opts(priv_data: *mut c_void) {
        let key = CString::new("preset").expect("CString conversion failed");
        let value = CString::new("ultrafast").expect("CString conversion failed");
        unsafe {
            sys::av_opt_set(priv_data, 
                key.as_ptr(), 
                value.as_ptr(),
                0
            );
        }
        
        let key = CString::new("crf").expect("CString conversion failed");
        let value = CString::new("0").expect("CString conversion failed");
        unsafe {
            sys::av_opt_set(priv_data, 
                key.as_ptr(), 
                value.as_ptr(),
                0
            );
        }
    }
}