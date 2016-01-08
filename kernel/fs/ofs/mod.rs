//! A module for os1 FS (OFS)

mod structs;

use io::block::BlockDevice;
use super::traits::FileSystem;
use self::structs::*;

/// A handle on the file system for
/// all needed operations.
pub struct OFS<B: BlockDevice> {
    device: B,
}

impl<B: BlockDevice> OFS<B> {
    // TODO
}

//TODO
//impl FileSystem for OFS {

//}
