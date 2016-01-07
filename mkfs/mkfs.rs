use std::io::prelude::*;
use std::fs::File;

const SECTOR_SIZE: usize = 512;

pub fn main() {
    // generate image
    let mut data: [u8; SECTOR_SIZE * 2] = [0; SECTOR_SIZE * 2];

    data[0] = 'O' as u8;
    data[1] = 'S' as u8;
    data[2] = 'F' as u8;
    
    // write to file
    let mut buffer = File::create("hdd.img").ok().expect("Could not create file hdd.img");
    let _ = buffer.write(&data);
}
