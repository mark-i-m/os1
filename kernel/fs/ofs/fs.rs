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
    pub fn open(&mut self, inode: usize) -> Result<File<B>, Error> {
        // lock the fs
        let mut fs = self.fs.down();

        if !fs.is_file(inode) {
            Err(Error::new("No such file or directory"))
        } else {
            let i = fs.get_inode(inode);
            let d = i.data;
            printf!("Open file {:?}: i {}, d {}\n", i.name, inode, d);
            Ok(File {
                inode_num: inode,
                inode: i,
                offset: 0,
                offset_dnode: d,
                ofs: self.fs.clone(),
            })
        }
    }

    /// Create a link (directed edge) from file `a` to file `b`. `a` and `b` are the inode number
    /// of the files.
    pub fn link(&mut self, a: usize, b: usize) -> Option<Error> {
        // TODO
        // TODO: make sure that they are not already linked
        Some(Error::new("OFS is read-only until I think about consistency..."))
    }

    /// Remove a link (directed edge) from file `a` to file `b`. `a` and `b` are the inode number
    /// of the files.
    pub fn unlink(&mut self, a: usize, b: usize) -> Option<Error> {
        // TODO
        // TODO: make sure that they are already linked
        Some(Error::new("OFS is read-only until I think about consistency..."))
    }

    /// Return metadata for the file with inode `a` or None if the file does not exist
    pub fn stat(&self, a: usize) -> Option<Inode> {
        let mut fs = self.fs.down();
        if fs.is_file(a) {
            Some(fs.get_inode(a))
        } else {
            None
        }
    }

    /// Create a new file and a link from `a` to it. `a` is the inode number of the file. Return
    /// the inode number of the new file.
    pub fn new_file(&mut self) -> Result<usize, Error> {
        return Err(Error::new("OFS is read-only until I think about consistency..."));

        let mut fs = self.fs.down();

        let inode_num = fs.get_new_inode();
        let dnode_num = fs.get_new_dnode();
        let inode = Inode {
            name: UNNAMED,
            uid: 0,
            gid: 0,
            user_perm: 7,
            group_perm: 0,
            all_perm: 0,
            flags: 0,
            size: 0,
            data: dnode_num,
            created: OFSDate::now(),
            modified: OFSDate::now(),
            links: [0; 22],
        };

        let mut tmp = BlockDataBuffer::new(mem::size_of::<Inode>());
        unsafe {
            *tmp.get_ref_mut(0) = inode;
        }

        let inode_start_offset = fs.get_inode_offset(inode_num);
        fs.device.write_fully(inode_start_offset, &mut tmp);

        // TODO: link the file to the the current working file of the creating process

        Ok(inode_num)
    }

    /// Delete file `a`. `a` is the inode number of the file.
    pub fn delete_file(&mut self, a: usize) -> Option<Error> {
        // TODO
        // Remove links
        // Remove Inode
        // Remove Dnodes
        // TODO: what if a file is deleted while another process is writing to it?
        Some(Error::new("OFS is read-only until I think about consistency..."))
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
    fn get_inode_offset(&self, i: usize) -> usize {
        let inode_sector = self.get_inode_start_sector() + i/Inode::inodes_per_sector();
        let inode_mod = i % Inode::inodes_per_sector();

        let inode_size = mem::size_of::<Inode>();
        inode_sector*SECTOR_SIZE+inode_mod*inode_size
    }

    /// Get the disk offset of the `d`th dnode.
    fn get_dnode_offset(&self, d: usize) -> usize {
        printf!("get dnode offset({:X})\n", d);
        let dnode_sector = self.get_dnode_start_sector() + d/Dnode::dnodes_per_sector();
        printf!("dnodes start @ {:X}, {:X}\n", self.get_dnode_start_sector(), dnode_sector);
        let dnode_mod = d % Dnode::dnodes_per_sector();

        let dnode_size = mem::size_of::<Dnode>();
        dnode_sector*SECTOR_SIZE+dnode_mod*dnode_size
    }

    /// Get the `i`th inode
    pub fn get_inode(&mut self, i: usize) -> Inode {
        let inode_sector = self.get_inode_start_sector() + i/Inode::inodes_per_sector();
        let inode_mod = i % Inode::inodes_per_sector();

        printf!("get_inode: {:X}, {:X}, {:X}\n", inode_sector, inode_mod, self.get_inode_start_sector());

        // read from disk
        let inode_size = mem::size_of::<Inode>();
        let mut buf = BlockDataBuffer::new(inode_size);
        self.device.read_fully(inode_sector*SECTOR_SIZE+inode_mod*inode_size, &mut buf);

        unsafe {
            buf.get_ref::<Inode>(0).clone()
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
            buf.get_ref::<Dnode>(0).clone()
        }
    }

    /// Allocate a new inode and return its index
    ///
    /// # Panics
    /// Panics if there is not a free inode
    fn get_new_inode(&mut self) -> usize {
        // read bitmap
        let mut buf = BlockDataBuffer::new(self.meta.num_inode / 8);
        self.device.read_fully(SECTOR_SIZE, &mut buf);

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

                        // writeback
                        buf.set_offset(i);
                        self.device.write_exactly(SECTOR_SIZE + i, 1, &mut buf);

                        // return dnode number
                        return i*8 + b
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
    fn get_new_dnode(&mut self) -> usize {
        // read bitmap
        let inode_bitmap_size = self.meta.num_inode / 8 / SECTOR_SIZE;

        let dnode_bitmap_start_sector = 1 + inode_bitmap_size;

        let mut buf = BlockDataBuffer::new(self.meta.num_dnode / 8);
        self.device.read_fully(dnode_bitmap_start_sector * SECTOR_SIZE, &mut buf);

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

                        // writeback
                        buf.set_offset(i);
                        self.device.write_exactly(dnode_bitmap_start_sector * SECTOR_SIZE + i, 1, &mut buf);

                        // return dnode number
                        return i*8 + b
                    }
                }
            }
        }

        panic!("Out of dnodes!");
    }

    /// Returns true if the file exists.
    pub fn is_file(&mut self, inode: usize) -> bool {
        // which byte of the bitmap
        let byte = inode / 8;

        // which bit of that byte
        let bit = inode % 8;

        // read bitmap
        let mut buf = BlockDataBuffer::new(1);
        self.device.read_fully(byte, &mut buf);

        let bitmap_byte = unsafe { *buf.get_ptr::<u8>(0) };

        bitmap_byte & (1 << bit) != 0
    }
}

impl<B: BlockDevice> File<B> {
    // TODO: change file name
    // TODO: define filename conflicts?

    /// Return the filename
    pub fn get_filename(&self) -> String {
        self.inode.get_filename()
    }

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

        printf!("dnode start: {:X}, offset: {:X}\n", dnode_start, dnode_offset);

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

    /// Write from the buffer at the buffer offset into the file at the file offset, overwriting
    /// any exisiting content. This method may extend the length of the file. This method might not
    /// write as much as possible from the file. This updates both the file offset and the buffer
    /// offset.
    fn write_part(&mut self, bytes: usize, buf: &mut BlockDataBuffer) -> usize {
        // TODO update modified time
        // TODO: what about concurrent writes?

        // lock the file system
        let mut fs = self.ofs.down();

        // How many bytes left in dnode?
        let dnode_size = mem::size_of::<Dnode>();

        // Where are we in the current dnode?
        let dnode_start = fs.get_dnode_offset(self.offset_dnode);
        let dnode_offset = self.offset % (dnode_size - 4);

        // Is this the last dnode of the file?
        let last_dnode = self.inode.size - self.offset <= dnode_size - dnode_offset;

        // Will the write stay within this dnode?
        let within = if last_dnode {
            dnode_size >= dnode_offset + bytes
        } else {
            dnode_size - 4 >= dnode_offset + bytes
        };

        // Do the write
        if within {
            let num_write = bytes;
            fs.device.write_exactly(dnode_start + dnode_offset, num_write, buf);
            let eof_offset = self.inode.size % (dnode_size - 4);
            if last_dnode && eof_offset < dnode_offset + num_write {
                self.inode.size += dnode_offset + num_write - eof_offset;
            }
            num_write
        } else {
            let num_write = min(dnode_size - 4 - dnode_offset, bytes);
            let curr_dnode = fs.get_dnode(self.offset_dnode);
            let tmp = &mut BlockDataBuffer::new(dnode_size);
            unsafe {
                copy((&curr_dnode.data[0]) as *const usize as *const u8, tmp.get_ptr::<u8>(0), dnode_size);
            }
            let next_dnode = if last_dnode {
                let new_dnode = fs.get_new_dnode();
                unsafe {
                    *tmp.get_ptr::<usize>((dnode_size/mem::size_of::<usize>()) - 1) = new_dnode;
                }
                self.inode.size += num_write + 1; // +1 so this is no longer the last dnode
                new_dnode
            } else {
                curr_dnode.get_next()
            };
            let old_buf_offset = buf.offset();
            unsafe {
                copy(buf.get_ptr::<u8>(old_buf_offset), tmp.get_ptr::<u8>(dnode_offset), num_write);
            }
            let dnode_start_offset = fs.get_dnode_offset(self.offset_dnode);
            fs.device.write_fully(dnode_start_offset, tmp);
            self.offset += num_write;
            self.offset_dnode = next_dnode;
            buf.set_offset(old_buf_offset + num_write);
            num_write
        }
    }

    /// Seek into the file to the given `offset`.
    pub fn seek(&mut self, mut offset: usize) {
        // TODO: optimizations

        if offset >= self.inode.size {
            // seek past EOF
            offset = self.inode.size;
        }

        let dnode_size = mem::size_of::<Dnode>();
        let dnode_offset = self.offset % (dnode_size - 4);
        let last_dnode = self.inode.size - self.offset <= dnode_size - dnode_offset;
        let dnode_rem = if last_dnode {
            dnode_size - dnode_offset
        } else {
            dnode_size - 4 - dnode_offset
        };

        if (offset > self.offset && offset - self.offset < dnode_rem) ||
           (offset < self.offset && self.offset - offset < dnode_offset) {
            // in the same dnode
            self.offset = offset;
        } else if offset > self.offset {
            // Seek forwards

            {
                // lock fs
                let mut fs = self.ofs.down();

                // get the next dnode
                let curr_dnode = fs.get_dnode(self.offset_dnode);
                let next_dnode = curr_dnode.get_next();

                self.offset += dnode_size - 4 - dnode_offset;
                self.offset_dnode = next_dnode;
            } // unlock

            self.seek(offset);
        } else if offset < self.offset {
            // Seek backwards
            self.offset = 0;
            self.offset_dnode = self.inode.data;

            self.seek(offset);
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

    /// Write `bytes` bytes from the buffer at the buffer offset to the file at the file offset,
    /// overwriting any existing content at the offset.
    /// This will increase the length of the file if necessary.  This updates both the file and
    /// buffer offsets.
    pub fn write(&mut self, bytes: usize, buf: &mut BlockDataBuffer) {
        panic!("OFS is read-only until I think about consistency...");
        let mut written = 0;
        while written < bytes {
            written += self.write_part(bytes - written, buf);
        }
    }
}

impl<B : BlockDevice> Drop for File<B> {
    /// Close the file. Write back the inode
    fn drop(&mut self) {
        //TODO: uncomment and correct
        //let mut fs = self.ofs.down();

        //let inode_start_offset = fs.get_inode_offset(self.inode_num);

        //let mut tmp = BlockDataBuffer::new(mem::size_of::<Inode>());
        //unsafe {
        //    *tmp.get_ptr(0) = self.inode.clone();
        //}

        //fs.device.write_fully(inode_start_offset, &mut tmp);
    }
}
