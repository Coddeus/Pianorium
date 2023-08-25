
fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=\"/usr/local/Cellar/sdl2/2.28.2/lib\"");
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-search=\"D:\\a\\Pianorium\\Pianorium\\SDL2-2.28.2\\x86_64-w64-mingw32\\lib\"");
}