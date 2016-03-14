//! A module for file system stuff

pub mod ofs;

use alloc::arc::Arc;
use alloc::boxed::Box;

use io::ide::IDE;
use io::block::BlockDevice;
use sync::Semaphore;
use self::ofs::fs::OFS;

static mut ROOT_FS: *mut OFS<IDE> = 0 as *mut OFS<IDE>;

/// Initialize the file system
pub fn init(mut device: IDE) {
    unsafe {
        let mut ofs = box OFS::new(Arc::new(Semaphore::new(device, 1)));
        ROOT_FS = Box::into_raw(ofs);
    }

    printf!("filesystem inited\n");
}
