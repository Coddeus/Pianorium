
fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-search=\"/usr/lib\"");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=\"/usr/local/Cellar/sdl2/2.28.2/lib\"");
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-search=\"./sdl2-2.28.2/lib\"");
}