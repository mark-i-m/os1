//! A module for file system stuff

pub mod ofs;

use alloc::arc::Arc;

use io::ide::IDE;
use self::ofs::fs::OFS;

static mut ROOT_FS: Option<OFS<IDE>> = None;

/// Initialize the file system
pub fn init(mut device: IDE) {
    unsafe {
        let mut ofs = OFS::new(device);
        ROOT_FS = Some(ofs);
    }

    printf!("filesystem inited\n");
}
