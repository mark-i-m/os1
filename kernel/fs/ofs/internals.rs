//! A module for OFS abstractions

use alloc::arc::Arc;

use core::cmp::min;
use core::mem;
use core::ptr::copy;

use sync::Semaphore;
use string::String;
use io::block::{BlockDevice, BlockDataBuffer};
use super::hw::*;
use super::super::error::Error;

// NOTE: for now, the number of inodes must be chosen so that the last inode just completes its
// block. That is, the first dnode comes immediately after the last inode *AND* at the start of the
// next block.
//
// NOTE: all sections of the volume must exactly fit in the space allocated to them. For example,
// all bitmaps must be an integer multiple of the block size.
//
// NOTE: for now all files must have a multiple of 4B

/// The OFS interface
pub struct OFS<B: BlockDevice> {
    // block device
    pub device: B,

    // metadata
    pub meta: Metadata,
}

impl<B: BlockDevice> OFS<B> {
    /// Get the block number of the dnode bitmap
    pub fn dnode_bitmap_block(&self) -> usize {
        let inode_blocks = self.meta.num_inode / 8 / self.device.get_block_size();
        1 + inode_blocks
    }

    /// Get the block number of the first inode
    pub fn first_inode_block(&self) -> usize {
        let blk_size = self.device.get_block_size();

        let inode_map_bytes = self.meta.num_inode / 8;
        let inode_map_sectors = if inode_map_bytes % blk_size > 0 {
            inode_map_bytes / blk_size + 1
        } else {
            inode_map_bytes / blk_size
        };

        let dnode_map_bytes = self.meta.num_dnode / 8;
        let dnode_map_sectors = if dnode_map_bytes % blk_size > 0 {
            dnode_map_bytes / blk_size + 1
        } else {
            dnode_map_bytes / blk_size
        };

        1 + inode_map_sectors + dnode_map_sectors
    }

    /// Get the block number of the first dnode
    pub fn first_dnode_block(&self) -> usize {
        self.first_inode_block() +
        self.meta.num_inode * mem::size_of::<Inode>() / self.device.get_block_size()
    }

    /// Get the block number of the `inode`th inode
    pub fn inode_num_to_block_num(&self, inode: usize) -> usize {
        self.first_inode_block() + inode * mem::size_of::<Inode>() / self.device.get_block_size()
    }

    /// Get the offset into its block of the `inode`th inode
    pub fn inode_num_to_block_offset(&self, inode: usize) -> usize {
        (inode % (self.device.get_block_size() / mem::size_of::<Inode>())) * mem::size_of::<Inode>()
    }

    /// Get the block number of the `dnode`th dnode
    pub fn dnode_num_to_block_num(&self, dnode: usize) -> usize {
        self.first_dnode_block() + dnode * mem::size_of::<Dnode>() / self.device.get_block_size()
    }

    /// Get the offset into its block of the `dnode`th dnode
    pub fn dnode_num_to_block_offset(&self, dnode: usize) -> usize {
        (dnode % (self.device.get_block_size() / mem::size_of::<Dnode>())) * mem::size_of::<Dnode>()
    }

    /// Get the `inode`th inode
    pub fn get_inode(&mut self, inode: usize) -> Inode {
        let block = self.inode_num_to_block_num(inode);
        let offset = self.inode_num_to_block_offset(inode);
        let size = mem::size_of::<Inode>();

        // read from disk
        let mut buf = BlockDataBuffer::new(size);
        self.device.read_fully(block, offset, &mut buf);

        unsafe { buf.get_ref::<Inode>(0).clone() }
    }

    /// Get the `dnode`th dnode
    pub fn get_dnode(&mut self, dnode: usize) -> Dnode {
        let block = self.dnode_num_to_block_num(dnode);
        let offset = self.dnode_num_to_block_offset(dnode);
        let size = mem::size_of::<Dnode>();

        // read from disk
        let mut buf = BlockDataBuffer::new(size);
        self.device.read_fully(block, offset, &mut buf);

        unsafe { buf.get_ref::<Dnode>(0).clone() }
    }

    /// Allocate a new inode and return its index
    ///
    /// # Panics
    /// Panics if there is not a free inode
    pub fn alloc_inode(&mut self) -> usize {
        // read bitmap (block 1, offset 0)
        let mut buf = BlockDataBuffer::new(self.meta.num_inode / 8);
        self.device.read_fully(1, 0, &mut buf);

        // find first free bit, set it, and return
        for i in 0..(self.meta.num_inode / 8) {
            let bitmap_byte = unsafe { *buf.get_ptr::<u8>(i) };
            if bitmap_byte != 0xFF {
                for b in 0..8 {
                    // found free bit
                    if bitmap_byte & (1 << b) == 0 {
                        // set bit
                        unsafe {
                            *buf.get_ptr::<u8>(i) |= 1 << b;
                        }

                        // TODO: what if the bitmap is more than 1 block?
                        // writeback
                        buf.set_offset(i);
                        self.device.write_exactly(1, i, 1, &mut buf);

                        // return dnode number
                        return i * 8 + b;
                    }
                }
            }
        }

        panic!("Out of inodes!");
    }

    /// Allocate a new dnode and return its index
    ///
    /// # Panics
    /// Panics if there is not a free dnode
    pub fn alloc_dnode(&mut self) -> usize {
        // read bitmap
        let dnode_bitmap_block = self.dnode_bitmap_block();
        let mut buf = BlockDataBuffer::new(self.meta.num_dnode / 8);
        self.device.read_fully(dnode_bitmap_block, 0, &mut buf);

        // find first free bit, set it, and return
        for i in 0..(self.meta.num_dnode / 8) {
            let bitmap_byte = unsafe { *buf.get_ptr::<u8>(i) };
            if bitmap_byte != 0xFF {
                for b in 0..8 {
                    // found free bit
                    if bitmap_byte & (1 << b) == 0 {
                        // set bit
                        unsafe {
                            *buf.get_ptr::<u8>(i) |= 1 << b;
                        }

                        // TODO: what if the bitmap is more than 1 block?
                        // writeback
                        buf.set_offset(i);
                        self.device
                            .write_exactly(dnode_bitmap_block, i, 1, &mut buf);

                        // return dnode number
                        return i * 8 + b;
                    }
                }
            }
        }

        panic!("Out of dnodes!");
    }

    pub fn free_inode(&mut self, inode: usize) {
        // TODO
    }

    pub fn free_dnode(&mut self, dnode: usize) {
        // TODO
    }

    pub fn is_free_inode(&mut self, inode: usize) -> bool {
        // which byte of the bitmap
        let byte = inode / 8;

        // which bit of that byte
        let bit = inode % 8;

        // read bitmap
        let mut buf = BlockDataBuffer::new(1);
        self.device.read_fully(1, byte, &mut buf);

        let bitmap_byte = unsafe { *buf.get_ptr::<u8>(0) };

        bitmap_byte & (1 << bit) == 0
    }

    pub fn mark_file_open(&mut self, inode: usize, read_only: bool) {
        // TODO
    }

    pub fn unmark_file_open(&mut self, inode: usize) {
        // TODO
    }
}
