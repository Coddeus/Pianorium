use std::fs::{create_dir, remove_dir_all, remove_file};

pub fn setup() -> std::io::Result<()>{
    let _ = remove_dir_all("temp");
    create_dir("temp")?;
    Ok(())
}

pub fn teardown() -> std::io::Result<()>{
    remove_dir_all("temp")?;
    remove_file("index.txt")?;
    remove_file("ffreport.log")?;
    remove_file("ffconcat.log")?;
    Ok(())
}