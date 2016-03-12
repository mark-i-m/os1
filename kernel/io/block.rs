//! A module for accessing hard disks

use alloc::heap;

use core::cmp::min;
use core::ptr::copy;
use core::mem;

/// A data structure for use with block devices The buffer has an internal pointer that can be used
/// for conveniently performing sequential writes or reads to the buffer.  Also, implementors of
/// this trait must also implement `Drop`, preventing memory leaks.
pub struct BlockDataBuffer {
    buf: *mut u8,
    size: usize,
    offset: usize,
}

/// An abstraction over block devices
pub trait BlockDevice {
    /// Get the block size of this block device in bytes
    fn get_block_size(&self) -> usize;

    /// Read the given block into the buffer
    ///
    /// # Panics
    ///
    /// The method panics if there is not enough space in the buffer
    fn read_block(&mut self, block_num: usize, buffer: &mut BlockDataBuffer);

    /// Write the given block from the buffer
    fn write_block(&mut self, block_num: usize, buffer: &BlockDataBuffer);

    /// Read from the block device at `offset` into the given buffer starting at the buffer's
    /// internal offset. This method will read no more data than will fit into the remaining space
    /// in the buffer, but it may also read less. It will update the buffer's offset, and return
    /// the number of bytes read.
    fn read(&mut self, offset: usize, buffer: &mut BlockDataBuffer) -> usize {
        // read the block where the data we want starts
        let blk_size = self.get_block_size();
        let sector = offset / blk_size;
        let mut block_buf = BlockDataBuffer::new(blk_size);
        self.read_block(sector, &mut block_buf);

        // get the portion we want
        let buf_offset = offset - (sector * blk_size);
        let num_read = min(buffer.size(), blk_size - buf_offset);
        unsafe {
            let buf = buffer.get_ptr::<u8>(buffer.offset());
            copy(block_buf.get_ptr::<u8>(buf_offset), buf, num_read);
        }

        // update the offset
        let new_offset = buffer.offset() + num_read;
        buffer.set_offset(new_offset);

        // return the number of bytes read
        num_read
    }

    /// Write from the buffer to the disk starting at the buffer's internal offset. This may not
    /// write the whole buffer. This will update the buffer's offset, and return the number of
    /// bytes written.
    fn write(&mut self, offset: usize, buffer: &BlockDataBuffer) -> usize {
        0 // TODO
    }

    /// Read from the block device at `offset` into the given buffer starting at the buffer's
    /// internal offset. This method will fill the remaining space in the buffer. It will update
    /// the buffer's offset.
    ///
    /// # Panics
    ///
    /// The method panics if there is not enough space in the buffer
    fn read_fully(&mut self, mut offset: usize, buffer: &mut BlockDataBuffer) {
        // find remaining space in the buffer
        let mut remaining = buffer.size() - buffer.offset();

        // read into the buffer until it is full
        while remaining > 0 {
            let read = self.read(offset, buffer);
            offset += read;
            remaining -= read;
        }
    }

    /// Write from the buffer to the disk starting at the buffer's internal offset. This will
    /// write the whole buffer. This will update the buffer's offset.
    fn write_fully(&mut self, mut offset: usize, buffer: &BlockDataBuffer) {
        // TODO
    }
}

impl BlockDataBuffer {
    /// Create a new buffer with the given capacity
    pub fn new(size: usize) -> BlockDataBuffer {
        unsafe {
            BlockDataBuffer {
                buf: heap::allocate(size, 1),
                size: size,
                offset: 0,
            }
        }
    }

    /// Get the internal offset in bytes into this buffer
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Set the internal offset in bytes into this buffer
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    /// Get the size of this buffer in bytes
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get a pointer of the given type to the
    /// offset in the buffer. The offset is relative to the
    /// size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ptr<T>(&self, offset: usize) -> *mut T {
        let t_size = mem::size_of::<T>();
        let num_ts = self.size() / t_size;

        if offset >= num_ts {
            panic!("Out of bounds");
        }

        self.buf.offset((offset * t_size) as isize) as *mut T
    }

    /// Get a reference of the given type to the offset
    /// in the buffer. The offset is relative to the
    /// size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ref<T>(&self, offset: usize) -> &T {
        &*self.get_ptr(offset)
    }

    /// Get a mutable reference of the given type to the offset
    /// in the buffer. The offset is relative to the
    /// size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ref_mut<T>(&self, offset: usize) -> &mut T {
        &mut *self.get_ptr(offset)
    }
}

impl Drop for BlockDataBuffer {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(self.buf, self.size, 1);
        }
    }
}
