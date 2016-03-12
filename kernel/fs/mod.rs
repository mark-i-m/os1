//! A module for file system stuff

pub mod ofs;

use alloc::arc::Arc;
use alloc::boxed::Box;

use io::ide::IDE;
use self::ofs::fs::OFS;

static mut ROOT_FS: *mut OFS<IDE> = 0 as *mut OFS<IDE>;

/// Initialize the file system
pub fn init(mut device: IDE) {
    unsafe {
        let mut ofs = box OFS::new(device);
        ROOT_FS = Box::into_raw(ofs);
    }

    printf!("filesystem inited\n");
}
