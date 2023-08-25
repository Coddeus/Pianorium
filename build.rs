
fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=\"/usr/local/Cellar/sdl2/2.28.2/lib\"");
}