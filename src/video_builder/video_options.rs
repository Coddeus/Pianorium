use std::collections::HashMap;
use ffmpeg_next::Rational;

#[derive(Clone)]
pub struct VideoOptions {
    pub output_path: String,
    pub metadata: HashMap<String, String>,

    pub video_time_base: Rational,
    pub video_codec: String,
    pub video_codec_params: HashMap<String, String>,
    pub pixel_format_in: String,
    pub pixel_format_out: String,
    pub resolution_in: (u32, u32),
    pub resolution_out: (u32, u32),
}