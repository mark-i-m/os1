//! A module for file system stuff

use io::ide::{IDE, IDEBuf};

pub mod ofs;

/// Initialize the file system
pub fn init(rootfs: IDE) {

    printf!("filesystem inited\n");
}
