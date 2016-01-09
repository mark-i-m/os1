use std::io::prelude::*;
use std::fs::File;
use std::mem::transmute;

// OFS constants
const SECTOR_SIZE: usize = 512;

const INODE_SIZE: usize = 64;
const DNODE_SIZE: usize = SECTOR_SIZE;

// Total size of sample disk designed
// to keep the math simple
const TOTAL_SIZE: usize = SECTOR_SIZE // meta data sector
                        + SECTOR_SIZE // inode bitmap
                        + SECTOR_SIZE // dnode bitmap
                        + SECTOR_SIZE*8 * INODE_SIZE // inodes
                        + SECTOR_SIZE*8 * DNODE_SIZE // dnodes
                        ;

// pub struct Inode {
//     name: [u8; 12],         // file name (up to 12B)
//     uid: usize,             // owner UID
//     gid: usize,             // group GID
//     size: usize,            // file size in bytes
//     data: usize,            // index of Dnode with contents
//     created: usize,         // date created
//     modified: usize,        // date last modified
//     parents: [usize; 3],    // indices of inodes pointing to this node
//                             //     upto 3, but the last can be a link
//                             //     to more pointers
//     children: [usize; 4],   // indices of inodes pointed to by this node
//                             //     upto 4, but the last can be a link
//                             //     to more pointers
// }

pub fn main() {
    // Generate image
    let mut data: [u32; TOTAL_SIZE/4] = [0; TOTAL_SIZE/4];

    // Meta data
    data[0] = 0x00_53_46_4F;            // magic: OSF\0
    data[1] = 0;                        // inode of root file
    data[2] = (SECTOR_SIZE*8) as u32;   // # inodes
    data[3] = (SECTOR_SIZE*8) as u32;   // # dnodes

    // Inode bitmap
    data[SECTOR_SIZE  /4] = 1 << 31;

    // Dnode bitmap
    data[SECTOR_SIZE*2/4] = 1 << 31;

    // Inodes
    let first_inode = SECTOR_SIZE*3/4;
    data[first_inode+0] = 0x2E_6F_6F_66;
    data[first_inode+1] = 0x00_74_78_74;
    data[first_inode+5] = 100;
    data[first_inode+6] = 0;
    data[first_inode+9] = 0;

    // Dnodes
    for i in 0..128 {
        data[SECTOR_SIZE/4*515+i] = 0xDEADBEAF;
    }

    // write to file
    let mut buffer = File::create("hdd.img").ok().expect("Could not create file hdd.img");
    unsafe {
        let _ = buffer.write(&transmute::<[u32; TOTAL_SIZE/4], [u8; TOTAL_SIZE]>(data));
    }
}
