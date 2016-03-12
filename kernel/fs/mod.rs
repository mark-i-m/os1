//! A module for file system stuff

use io::ide::{IDE, IDEBuf};
use io::block::{BlockDevice, BlockDataBuffer};

pub mod ofs;

/// Initialize the file system
pub fn init(mut rootfs: IDE) {

    let mut buf = IDEBuf::new(512);    

    rootfs.read_block(0, &mut buf);

    unsafe {
        if *buf.get_ref::<u8>(0) != 'O' as u8 ||
           *buf.get_ref::<u8>(1) != 'S' as u8 || 
           *buf.get_ref::<u8>(2) != 'F' as u8 || 
           *buf.get_ref::<u8>(3) != '\x00' as u8 {
            panic!("Oh no!");
           }
    }

    unsafe {
        *buf.get_ref_mut::<u8>(10) = 'A' as u8;
        *buf.get_ref_mut::<u8>(11) = 'b' as u8;
        *buf.get_ref_mut::<u8>(12) = 'C' as u8;
        *buf.get_ref_mut::<u8>(13) = 'd' as u8;
    }

    rootfs.write_block(0, &buf);

    printf!("filesystem inited\n");
}
