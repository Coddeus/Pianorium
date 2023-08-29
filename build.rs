fn main() {
    #[cfg(all(windows, target_env = "msvc"))]
    std::env::set_var("FFMPEG_DIR", "./ffmpeg");
}