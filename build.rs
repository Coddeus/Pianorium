fn main() {
    vcpkg::Config::new()
        .target_triplet("x64-windows-static-md")
        .find_package("sdl2")
        .unwrap();
    vcpkg::Config::new()
        .target_triplet("x64-windows-static-md")
        .find_package("ffmpeg")
        .unwrap();
}