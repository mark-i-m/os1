use alloc::arc::Arc;

use core::cmp::min;
use core::mem;
use core::ptr::copy;

use io::block::{BlockDevice, BlockDataBuffer};
use string::String;
use sync::Semaphore;
use super::internals::*;
use super::hw::*;

/// A handle on the file for all needed operations.
pub struct File<B: BlockDevice> {
    // TODO: make these private
    // TODO: need marker if this file is read_only
    pub inode_num: usize,
    pub inode: Inode,
    pub offset: usize,
    pub offset_dnode: usize,
    pub ofs: Arc<Semaphore<OFS<B>>>,
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

        let dnode_size = mem::size_of::<Dnode>();

        // Where are we in the current dnode?
        // How many bytes left in dnode?
        let dnode = fs.dnode_num_to_block_num(self.offset_dnode);
        let dnode_offset = self.offset % (dnode_size - 4);

        // Is this the last dnode of the file?
        let last_dnode = self.inode.size - self.offset <= dnode_size - dnode_offset;

        if last_dnode {
            let bytes_left = self.inode.size - self.offset;
            let num_read = min(bytes_left, buf.size() - buf.offset());
            fs.device.read_exactly(dnode, dnode_offset, bytes_left, buf);
            self.offset += num_read;
            num_read
        } else {
            let bytes_left = dnode_size - (self.offset % (dnode_size - 4)) - 4;

            if buf.size() - buf.offset() < bytes_left {
                let num_read = buf.size() - buf.offset();
                fs.device.read_exactly(dnode, dnode_offset, num_read, buf);
                self.offset += num_read;
                num_read
            } else {
                let tmp = &mut BlockDataBuffer::new(bytes_left + 4);
                fs.device.read_fully(dnode, dnode_offset, tmp);
                self.offset += bytes_left;
                self.offset_dnode = unsafe { *tmp.get_ptr(bytes_left / 4) };
                let buf_offset = buf.offset();
                unsafe {
                    copy(tmp.get_ptr::<u8>(0),
                         buf.get_ptr::<u8>(buf_offset),
                         bytes_left);
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

        // lock the file system
        let mut fs = self.ofs.down();

        // How many bytes left in dnode?
        let dnode_size = mem::size_of::<Dnode>();

        // Where are we in the current dnode?
        let dnode = fs.dnode_num_to_block_num(self.offset_dnode);
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
            fs.device.write_exactly(dnode, dnode_offset, num_write, buf);
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
                copy((&curr_dnode.data[0]) as *const usize as *const u8,
                     tmp.get_ptr::<u8>(0),
                     dnode_size);
            }
            let next_dnode = if last_dnode {
                let new_dnode = fs.alloc_dnode();
                unsafe {
                    *tmp.get_ptr::<usize>((dnode_size / mem::size_of::<usize>()) - 1) = new_dnode;
                }
                self.inode.size += num_write + 1; // +1 so this is no longer the last dnode
                new_dnode
            } else {
                curr_dnode.get_next()
            };
            let old_buf_offset = buf.offset();
            unsafe {
                copy(buf.get_ptr::<u8>(old_buf_offset),
                     tmp.get_ptr::<u8>(dnode_offset),
                     num_write);
            }
            let dnode_blk = fs.dnode_num_to_block_num(self.offset_dnode);
            let dnode_offset = fs.dnode_num_to_block_offset(self.offset_dnode);
            fs.device.write_fully(dnode_blk, dnode_offset, tmp);
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

    fn get_next_dnode() {
        // TODO
    }

    fn get_prev_dnode() {
        // TODO
    }

    fn add_link() {
        // TODO
    }

    fn remove_link() {
        // TODO
    }

    fn grow() {
        // TODO
    }

    fn change_metadata() {
        // TODO
    }

    // TODO: how to access metadata?
    /// Return the filename
    pub fn get_filename(&self) -> String {
        self.inode.get_filename()
    }
}

impl<B: BlockDevice> Drop for File<B> {
    /// Close the file. Write back the inode
    fn drop(&mut self) {
        // TODO: should only write back if opened as writeable
        // let mut fs = self.ofs.down();

        // let inode_start_offset = fs.get_inode_offset(self.inode_num);

        // let mut tmp = BlockDataBuffer::new(mem::size_of::<Inode>());
        // unsafe {
        //    *tmp.get_ptr(0) = self.inode.clone();
        // }

        // fs.device.write_fully(inode_start_offset, &mut tmp);
    }
}
