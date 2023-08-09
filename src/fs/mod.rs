use std::fs::{create_dir, remove_dir_all, remove_file};

pub fn setup() {
    match remove_dir_all("temp"){
        _ => {}
    };
    create_dir("temp").unwrap();
}

pub fn teardown() {
    match remove_dir_all("temp"){
        _ => {}
    };
    remove_file("index.txt").unwrap();
    remove_file("ffreport.log").unwrap();
    remove_file("ffconcat.log").unwrap();
}