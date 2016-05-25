use std::io::prelude::*;
use std::fs::File;
use std::mem::transmute;

// OFS constants
const SECTOR_SIZE: usize = 512; // bytes

const INODE_SIZE: usize = 128; // bytes
const DNODE_SIZE: usize = 512; // bytes

// Total size of sample disk designed
// to keep the math simple
const TOTAL_SIZE: usize = SECTOR_SIZE // meta data sector
                        + SECTOR_SIZE // inode bitmap
                        + SECTOR_SIZE // dnode bitmap
                        + SECTOR_SIZE*8 * INODE_SIZE // inodes
                        + SECTOR_SIZE*8 * DNODE_SIZE // dnodes
                        ;


//pub struct Inode {
//    pub name: [u8; 12],         // file name (up to 12B)
//    pub uid: usize,             // owner UID
//    pub gid: usize,             // group GID
//    pub user_perm: u8,          // user permissions
//    pub group_perm: u8,         // group permissions
//    pub all_perm: u8,           // everyone permissions
//    pub flags:u8,               // various flags
//    pub size: usize,            // file size in bytes
//    pub data: usize,            // index of Dnode with contents
//    pub created: OFSDate,       // date created
//    pub modified: OFSDate,      // date last modified
//    pub parents: [usize; 10],
//    pub children: [usize; 12],
//}

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
    data[SECTOR_SIZE*2/4] = 1 << 31 | 1 << 30;

    // Inodes
    let first_inode = SECTOR_SIZE*3/4;
    data[first_inode+0] = 0x2E_6F_6F_66;
    data[first_inode+1] = 0x00_74_78_74;
    data[first_inode+6] = 516; // size
    data[first_inode+7] = 1;

    // Dnodes
    let first_dnode = (3 * SECTOR_SIZE + SECTOR_SIZE * 8 * INODE_SIZE)/4;
    data[first_dnode] = 0x0000_BEB0;
    data[first_dnode+1] = 0xDEADBEBE;
    data[first_dnode+DNODE_SIZE/4] = 0xDEADBEEF;
    data[first_dnode+2*DNODE_SIZE/4-1] = 0;
    data[first_dnode+2*DNODE_SIZE/4-2] = 0xDEAD_0000;
    data[first_dnode+DNODE_SIZE/4+8] = 1; //32B into the file
    data[first_dnode+DNODE_SIZE/4+9] = 11; //36B into the file

    // write to file
    let mut buffer = File::create("hdd.img").ok().expect("Could not create file hdd.img");
    unsafe {
        let _ = buffer.write(&transmute::<[u32; TOTAL_SIZE/4], [u8; TOTAL_SIZE]>(data));
    }
}
