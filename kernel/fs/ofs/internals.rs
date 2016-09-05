//! A module for OFS abstractions

use alloc::arc::Arc;

use core::cmp::min;
use core::mem;
use core::ptr::copy;

use sync::Semaphore;
use string::String;
use io::block::{BlockDevice, BlockDataBuffer};
use io::ide::SECTOR_SIZE;
use super::hw::*;
use super::super::error::Error;

// TODO: file system consistency
// TODO: check permissions

/// The OFS interface
pub struct OFS<B: BlockDevice> {
    // block device
    device: B,

    // metadata
    meta: Metadata,
}

impl<B: BlockDevice> OFS<B> {
    pub fn inode_num_to_block_num(&self, inode: usize) -> usize {}

    pub fn dnode_num_to_block_num(&self, dnode: usize) -> usize {}

    pub fn get_inode(&self, inode: usize) -> Inode {}

    pub fn get_dnode(&self, dnode: usize) -> Dnode {}

    pub fn alloc_inode(&mut self) -> usize {}

    pub fn alloc_dnode(&mut self) -> usize {}

    pub fn free_inode(&mut self, inode: usize) {}

    pub fn free_dnode(&mut self, dnode: usize) {}

    pub fn mark_file_open(&mut self, inode: usize, read_only: bool) {}

    pub fn unmark_file_open(&mut self, inode: usize) {}
}
