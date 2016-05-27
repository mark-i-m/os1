//! A module for file system stuff

pub mod ofs;
pub mod error;

use alloc::boxed::Box;

use io::ide::IDE;
use self::ofs::fs::OFSHandle;

pub static mut ROOT_FS: *mut OFSHandle<IDE> = 0 as *mut OFSHandle<IDE>;

/// Initialize the root file system from the given device
pub fn init(device: IDE) {
    unsafe {
        let ofs = box OFSHandle::new(device);
        ROOT_FS = Box::into_raw(ofs);
    }

    printf!("filesystem inited\n");
}
