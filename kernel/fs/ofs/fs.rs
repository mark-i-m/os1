//! A module for OFS abstractions

use alloc::arc::Arc;

use core::mem;

use sync::StaticSemaphore;
use io::block::{BlockDevice, BlockDataBuffer};
use io::ide::{IDE, SECTOR_SIZE};
use super::hw::*;

/// A handle on the file system for all needed operations.
pub struct OFS<B: BlockDevice> {
    device: Arc<B>,

    // metadata
    meta: Metadata,

    // lock
    lock: StaticSemaphore,
}

/// A handle on the file for all needed operations.
pub struct File<B: BlockDevice> {
    inode_num: usize,
    inode: Inode,
    offset: usize,
    device: Arc<B>,
}

impl<B: BlockDevice> OFS<B> {
    /// Create a new handle on the device using the device
    pub fn new(mut device: B) -> OFS<B> {
        let mut buf = BlockDataBuffer::new(SECTOR_SIZE);

        device.read_fully(0, &mut buf);

        // get the metadata sector
        let meta = unsafe { buf.get_ref::<Metadata>(0) };

        // check the magic
        if &*meta.get_magic_str() != "OFS\x00" {
            panic!("This is not an OFS volume!");
        }

        OFS {
            device: device,
            meta: meta.clone(),
            lock: StaticSemaphore::new(1),
        }
    }

    /// Open the file with the given inode number and
    /// return a handle to it.
    pub fn open(&mut self, inode: usize) -> File<B> {
        File {
            inode_num: inode,
            inode: self.get_inode(inode),
        }
    }

    /// Get the sector number of the first inode
    fn get_inode_start_sector(&self) -> usize {
        let inode_map_bytes = self.meta.num_inode / 8;
        let inode_map_sectors = if inode_map_bytes % SECTOR_SIZE > 0 {
            inode_map_bytes / SECTOR_SIZE + 1
        } else {
            inode_map_bytes / SECTOR_SIZE
        };

        let dnode_map_bytes = self.meta.num_dnode / 8;
        let dnode_map_sectors = if dnode_map_bytes % SECTOR_SIZE > 0 {
            dnode_map_bytes / SECTOR_SIZE + 1
        } else {
            dnode_map_bytes / SECTOR_SIZE
        };

        1 + inode_map_sectors + dnode_map_sectors
    }

    /// Get the sector number of the first dnode
    fn get_dnode_start_sector(&self) -> usize {
        self.get_inode_start_sector() + self.meta.num_inode
    }

    /// Get the `i`th inode
    pub fn get_inode(&mut self, i: usize) -> Inode {
        let inode_sector = self.get_inode_start_sector() + i/Inode::inodes_per_sector();
        let inode_mod = i % Inode::inodes_per_sector();

        // read from disk
        let inode_size = mem::size_of::<Inode>();
        let mut buf = BlockDataBuffer::new(inode_size);
        self.device.read_fully(inode_sector*SECTOR_SIZE+inode_mod*inode_size, &mut buf);

        unsafe {
            buf.get_ref::<Inode>(inode_mod).clone()
        }
    }

    /// Get the `d`th dnode
    fn get_dnode(&mut self, d: usize) -> Dnode {
        let dnode_sector = self.get_dnode_start_sector() + d/Dnode::dnodes_per_sector();
        let dnode_mod = d % Dnode::dnodes_per_sector();

        // read from disk
        let dnode_size = mem::size_of::<Dnode>();
        let mut buf = BlockDataBuffer::new(dnode_size);
        self.device.read_fully(dnode_sector*SECTOR_SIZE+dnode_mod*dnode_size, &mut buf);

        unsafe {
            buf.get_ref::<Dnode>(dnode_mod).clone()
        }
    }
}

impl<B: BlockDevice> File<B> {
    // TODO: fix this

    pub fn read(&mut self) -> usize {

    }
}
