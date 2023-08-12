use std::fs::{create_dir, remove_dir_all, remove_file};

pub fn setup() -> std::io::Result<()>{
    let _ = remove_dir_all("temp");
    create_dir("temp")?;
    Ok(())
}

pub fn teardown() -> std::io::Result<()>{
    remove_dir_all("temp")?;
    let _ = remove_file("index.txt");
    let _ = remove_file("ff_concat_mp4.log");
    let _ = remove_file("ff_export_mp4.log");
    let _ = remove_file("ff_export_png.log");
    Ok(())
}