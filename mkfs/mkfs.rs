//! A simple tool for building OFS images

#![feature(braced_empty_structs, convert, read_exact, str_char)]

mod image;

use std::env::args;

use image::OFSImage;

pub fn main() {
    // read command line args
    let mut args = args();
    args.next(); // drop the first arg (./mkfs)

    // create a new image
    let mut image = OFSImage::new(image::SECTOR_SIZE*8, image::SECTOR_SIZE*8);

    // add the files to the image
    for arg in args {
        image.add_file(arg.as_str());
    }

    image.burn("hdd.img");

    //// Generate image
    //let mut data: [u32; TOTAL_SIZE/4] = [0; TOTAL_SIZE/4];

    //// Meta data
    //data[0] = 0x00_53_46_4F;            // magic: OSF\0
    //data[1] = (SECTOR_SIZE*8) as u32;   // # inodes
    //data[2] = (SECTOR_SIZE*8) as u32;   // # dnodes

    //// Inode bitmap
    //data[SECTOR_SIZE  /4] = 1 << 31;

    //// Dnode bitmap
    //data[SECTOR_SIZE*2/4] = 1 << 31 | 1 << 30;

    //// Inodes
    //let first_inode = SECTOR_SIZE*3/4;
    //data[first_inode+0] = 0x2E_6F_6F_66;
    //data[first_inode+1] = 0x00_74_78_74;
    //data[first_inode+6] = 516; // size
    //data[first_inode+7] = 1;

    //// Dnodes
    //let first_dnode = (3 * SECTOR_SIZE + SECTOR_SIZE * 8 * INODE_SIZE)/4;
    //data[first_dnode] = 0x0000_BEB0;
    //data[first_dnode+1] = 0xDEADBEBE;
    //data[first_dnode+DNODE_SIZE/4] = 0xDEADBEEF;
    //data[first_dnode+2*DNODE_SIZE/4-1] = 0;
    //data[first_dnode+2*DNODE_SIZE/4-2] = 0xDEAD_0000;
    //data[first_dnode+DNODE_SIZE/4+8] = 1; //32B into the file
    //data[first_dnode+DNODE_SIZE/4+9] = 11; //36B into the file

    //// write to file
    //let mut buffer = File::create("hdd.img").ok().expect("Could not create file hdd.img");
    //unsafe {
    //    let _ = buffer.write(&transmute::<[u32; TOTAL_SIZE/4], [u8; TOTAL_SIZE]>(data));
    //}
}
