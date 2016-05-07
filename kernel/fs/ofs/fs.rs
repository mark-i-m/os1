//! A module for OFS abstractions

use alloc::arc::Arc;

use core::cmp::min;
use core::mem;
use core::ptr::copy;

use sync::Semaphore;
use io::block::{BlockDevice, BlockDataBuffer};
use io::ide::SECTOR_SIZE;
use super::hw::*;

/// A safe handle on the file system for all needed operations.
pub struct OFSHandle<B: BlockDevice> {
    fs: Arc<Semaphore<OFS<B>>>,
}

/// The OFS interface
pub struct OFS<B: BlockDevice> {
    // block device
    device: B,

    // metadata
    meta: Metadata,
}

/// A handle on the file for all needed operations.
pub struct File<B: BlockDevice> {
    inode_num: usize,
    inode: Inode,
    offset: usize,
    offset_dnode: usize,
    ofs: Arc<Semaphore<OFS<B>>>,
}

impl<B: BlockDevice> OFSHandle<B> {
    /// Create a new handle on the fs using the device
    ///
    /// NOTE: we do not have to lock the device now, because ownership transfers to the handle :D
    pub fn new(mut device: B) -> OFSHandle<B> {
        let mut buf = BlockDataBuffer::new(SECTOR_SIZE);

        device.read_fully(0, &mut buf);

        // get the metadata sector
        let meta = unsafe { buf.get_ref::<Metadata>(0).clone() };

        // check the magic
        if &*meta.get_magic_str() != "OFS\x00" {
            panic!("This is not an OFS volume!");
        }

        OFSHandle {
            fs: Arc::new(Semaphore::new(OFS {
                device: device,
                meta: meta,
            }, 1)),
        }
    }

    /// Open the file with the given inode number and return a handle to it.
    pub fn open(&mut self, inode: usize) -> File<B> {
        // lock the fs
        let mut fs = self.fs.down();

        let i = fs.get_inode(inode);
        let d = i.data;
        //printf!("Open file: i {}, d {}\n", inode, d);
        File {
            inode_num: inode,
            inode: i,
            offset: 0,
            offset_dnode: d,
            ofs: self.fs.clone(),
        }
    }

    /// Create a link (directed edge) from file `a` to file `b`. `a` and `b` are the inode number
    /// of the files.
    pub fn link(&mut self, a: usize, b: usize) {
        // TODO
    }

    /// Return metadata for file `a`. `a` is the inode number of the file.
    pub fn stat(&self, a: usize) {
        // TODO
    }

    /// Create a new file and a link from `a` to it. `a` is the inode number of the file. Return
    /// the inode number of the new file.
    pub fn new_file(&mut self, a: usize) -> usize {
        // TODO
        0
    }

    /// Delete file `a`. `a` is the inode number of the file.
    pub fn delete_file(&mut self, a: usize) {
        // TODO
    }
}

impl<B: BlockDevice> OFS<B> {
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

    /// Get the sector number of the first dnode.
    /// 
    /// NOTE: Assume that the first dnode starts at the beginning of a sector
    fn get_dnode_start_sector(&self) -> usize {
        self.get_inode_start_sector() + self.meta.num_inode/Inode::inodes_per_sector()
    }

    /// Get the disk offset of the `i`th inode.
    fn get_inode_offset(&mut self, i: usize) -> usize {
        let inode_sector = self.get_inode_start_sector() + i/Inode::inodes_per_sector();
        let inode_mod = i % Inode::inodes_per_sector();

        let inode_size = mem::size_of::<Inode>();
        inode_sector*SECTOR_SIZE+inode_mod*inode_size
    }

    /// Get the disk offset of the `d`th dnode.
    fn get_dnode_offset(&self, d: usize) -> usize {
        let dnode_sector = self.get_dnode_start_sector() + d/Dnode::dnodes_per_sector();
        let dnode_mod = d % Dnode::dnodes_per_sector();

        let dnode_size = mem::size_of::<Dnode>();
        dnode_sector*SECTOR_SIZE+dnode_mod*dnode_size
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
    /// Read from the file at the file offset into the buffer at the buffer offset.  This method
    /// will not overflow the buffer or read past the end of the file, but it might not read as
    /// much as possible from the file, even if the buffer is not full. This updates both the
    /// file offset and the buffer offset.
    fn read_part(&mut self, buf: &mut BlockDataBuffer) -> usize {
        // EOF
        if self.inode.size == self.offset {
            return 0;
        }

        // lock the file system
        let mut fs = self.ofs.down();

        // How many bytes left in dnode?
        let dnode_size = mem::size_of::<Dnode>();

        // Where are we in the current dnode?
        let dnode_start = fs.get_dnode_offset(self.offset_dnode);
        let dnode_offset = self.offset % (dnode_size - 4);

        // Is this the last dnode of the file?
        let last_dnode = self.inode.size - self.offset <= dnode_size - dnode_offset;

        if last_dnode {
            let bytes_left = self.inode.size - self.offset;
            let num_read = min(bytes_left, buf.size() - buf.offset());
            fs.device.read_exactly(dnode_start + dnode_offset, bytes_left, buf);
            self.offset += num_read;
            num_read
        } else {
            let bytes_left = dnode_size - (self.offset % (dnode_size - 4)) - 4;

            if buf.size() - buf.offset() < bytes_left {
                let num_read = buf.size() - buf.offset();
                fs.device.read_exactly(dnode_start + dnode_offset, num_read, buf);
                self.offset += num_read;
                num_read
            } else {
                let tmp = &mut BlockDataBuffer::new(bytes_left + 4);
                fs.device.read_fully(dnode_start + dnode_offset, tmp);
                self.offset += bytes_left;
                self.offset_dnode = unsafe {*tmp.get_ptr(bytes_left/4)};
                let buf_offset = buf.offset();
                unsafe {
                    copy(tmp.get_ptr::<u8>(0), buf.get_ptr::<u8>(buf_offset), bytes_left);
                }
                buf.set_offset(buf_offset + bytes_left);
                bytes_left
            }
        }
    }

    /// Seek into the file to the given `offset`.
    pub fn seek(&mut self, offset: usize) {
        if offset > self.offset {
            //TODO
            // Seek forwards
        } else if offset < self.offset {
            //TODO
            // Seek backwards
        }
    }

    /// Fill the buffer starting at the buffer offset from the file starting at the file offset.
    /// This reads as much as possible from the file without overflowing the buffer or reading past
    /// the EOF. This updates both the file and buffer offsets.
    pub fn read(&mut self, buf: &mut BlockDataBuffer) -> usize {
        // EOF
        if self.inode.size == self.offset {
            return 0;
        }

        // How many bytes can fit in buf?
        let max_bytes = buf.size() - buf.offset();

        // How many bytes left in the file?
        let bytes_left = self.inode.size - self.offset;

        // Number of bytes we can read to buf
        let num_read = min(max_bytes, bytes_left);

        let new_offset = buf.offset() + num_read;

        while buf.offset() < new_offset {
            self.read_part(buf);
        }

        num_read
    }

    /// Write `bytes` bytes from the buffer at the buffer offset to the file at the file offset.
    /// This will increase the length of the file if necessary.  This updates both the file and
    /// buffer offsets.
    pub fn write(&mut self, bytes: usize, buf: &mut BlockDataBuffer) {
        //TODO
    }
}

impl<B : BlockDevice> Drop for File<B> {
    /// Close the file.
    fn drop(&mut self) {
        // TODO: close file
    }
}
