use std::{
    fs::File,
    io::{BufRead, BufReader},
    process, thread,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("tests/1.txt").unwrap();

    Ok(())
}

pub fn read_file_iter() {
    let file = File::open("tests/1.txt").unwrap();
    let buf_reader = BufReader::new(file);
    for d in buf_reader.lines() {
        println!("{:?}", d);
    }
}
