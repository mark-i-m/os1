//! A module for accessing hard disks

use alloc::Vec;

use core::borrow::{Borrow, BorrowMut};
use core::cmp::min;
use core::ptr::copy;
use core::mem;

/// A data structure for use with block devices.
///
/// The buffer has an internal pointer that can be used for conveniently performing sequential
/// writes or reads to the buffer.  Also, implementors of this trait must also implement `Drop`,
/// preventing memory leaks.
pub struct BlockDataBuffer {
    buf: Vec<u8>,
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

    /// Read from the specified block at `offset` into the given buffer starting at the buffer's
    /// offset. This method will read no more data than will fit into the remaining space in the
    /// buffer, but it may also read less. It will update the buffer's offset, and return the
    /// number of bytes read.
    ///
    /// # Panics
    ///
    /// The method panics if `offset >= self.get_block_size()`.
    fn read(&mut self, block_num: usize, offset: usize, buffer: &mut BlockDataBuffer) -> usize {
        printf!("Read: {:X} {:x}\n", block_num, offset);

        // read the block where the data we want starts
        let blk_size = self.get_block_size();

        if offset >= self.get_block_size() {
            panic!("Attempt to use block offset larger than block: {}", offset);
        }

        let mut block_buf = BlockDataBuffer::new(blk_size);
        self.read_block(block_num, &mut block_buf);

        // get the portion we want
        let num_read = min(buffer.size() - buffer.offset(), blk_size - offset);
        unsafe {
            let boffset = buffer.offset();
            let buf = buffer.get_ptr_mut::<u8>(boffset);
            copy(block_buf.get_ptr::<u8>(offset), buf, num_read);
        }

        // update the offset
        let new_offset = buffer.offset() + num_read;
        buffer.set_offset(new_offset);

        // return the number of bytes read
        num_read
    }

    /// Write from the buffer to the disk starting at the buffer's offset. This may not write the
    /// whole buffer. This will update the buffer's offset, and return the number of bytes written.
    ///
    /// # Panics
    ///
    /// The method panics if `offset >= self.get_block_size()`.
    fn write(&mut self, block_num: usize, offset: usize, buffer: &mut BlockDataBuffer) -> usize {
        // read the block we are about to modify
        let blk_size = self.get_block_size();

        if offset >= self.get_block_size() {
            panic!("Attempt to use block offset larger than block: {}", offset);
        }

        let mut block_buf = BlockDataBuffer::new(blk_size);
        self.read_block(block_num, &mut block_buf);

        // modify the block
        let num_written = min(buffer.size() - buffer.offset(), blk_size - offset);

        unsafe {
            let boffset = buffer.offset();
            let buf = buffer.get_ptr::<u8>(boffset);
            copy(buf, block_buf.get_ptr_mut::<u8>(offset), num_written);
        }

        // write modified block to disk
        self.write_block(block_num, &block_buf);

        // update the offset
        let new_offset = buffer.offset() + num_written;
        buffer.set_offset(new_offset);

        // return the number of bytes read
        num_written
    }

    /// Read from the specified block at `offset` into the given buffer starting at the buffer's
    /// offset. This method will fill the remaining space in the buffer. It will update the
    /// buffer's offset.
    ///
    /// # Panics
    ///
    /// The method panics if
    /// - there is not enough space in the buffer
    /// - `offset >= self.get_block_size()`
    fn read_fully(&mut self, block_num: usize, offset: usize, buffer: &mut BlockDataBuffer) {
        // find remaining space in the buffer
        let mut remaining = buffer.size() - buffer.offset();
        let mut so_far = 0;

        let blk_size = self.get_block_size();

        // read into the buffer until it is full
        while remaining > 0 {
            let read = self.read(block_num + so_far / blk_size, offset + so_far, buffer);
            remaining -= read;
            so_far += read;
        }
    }

    /// Write from the buffer to the disk starting at the buffer's internal offset. This will
    /// write the whole buffer. This will update the buffer's offset.
    ///
    /// # Panics
    ///
    /// The method panics if `offset >= self.get_block_size()`.
    fn write_fully(&mut self, block_num: usize, offset: usize, buffer: &mut BlockDataBuffer) {
        // find remaining space in the buffer
        let mut remaining = buffer.size() - buffer.offset();
        let mut so_far = 0;

        let blk_size = self.get_block_size();

        // read into the buffer until it is full
        while remaining > 0 {
            let written = self.write(block_num + so_far / blk_size, offset + so_far, buffer);
            remaining -= written;
            so_far += written;
        }
    }

    /// Read `bytes` bytes from the block device at `offset` into the given buffer starting at the
    /// buffer's internal offset. This method will fill the remaining space in the buffer. It will
    /// update the buffer's offset.
    ///
    /// # Panics
    ///
    /// The method panics if
    /// - there is not enough space in the buffer
    /// - `offset >= self.get_block_size()`
    fn read_exactly(&mut self,
                    block_num: usize,
                    offset: usize,
                    bytes: usize,
                    buffer: &mut BlockDataBuffer) {
        // TODO: make this more efficient
        let tmp = &mut BlockDataBuffer::new(bytes);
        self.read_fully(block_num, offset, tmp);
        unsafe {
            let buf_offset = buffer.offset();
            copy(tmp.get_ptr::<u8>(0),
                 buffer.get_ptr_mut::<u8>(buf_offset),
                 bytes);
            buffer.set_offset(buf_offset + bytes);
        }
    }

    /// Write `bytes` bytes to the block device at `offset` from the given buffer starting at the
    /// buffer's internal offset. This method will overwrite the existing disk contents. It will
    /// update the buffer's offset.
    fn write_exactly(&mut self,
                     block_num: usize,
                     offset: usize,
                     bytes: usize,
                     buffer: &mut BlockDataBuffer) {
        // TODO: make this more efficient
        let tmp = &mut BlockDataBuffer::new(bytes);
        unsafe {
            let buf_offset = buffer.offset();
            copy(buffer.get_ptr::<u8>(buf_offset),
                 tmp.get_ptr_mut::<u8>(0),
                 bytes);
            buffer.set_offset(buf_offset + bytes);
        }
        self.write_fully(block_num, offset, tmp);
    }
}

impl BlockDataBuffer {
    /// Create a new buffer with the given capacity
    pub fn new(size: usize) -> BlockDataBuffer {
        BlockDataBuffer {
            buf: Vec::with_capacity(size),
            size: size,
            offset: 0,
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

    /// Get a pointer of the given type to the offset in the buffer. The offset is relative to the
    /// size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ptr<T>(&self, offset: usize) -> *const T {
        let t_size = mem::size_of::<T>();
        let num_ts = self.size() / t_size;

        if offset >= num_ts {
            panic!("Out of bounds {} * {} out of {}",
                   offset,
                   t_size,
                   self.size());
        }

        let slice: &[u8] = self.buf.borrow();
        slice.as_ptr().offset((offset * t_size) as isize) as *const T
    }

    /// Get a mutable pointer of the given type to the offset in the buffer. The offset is relative
    /// to the size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ptr_mut<T>(&mut self, offset: usize) -> *mut T {
        let t_size = mem::size_of::<T>();
        let num_ts = self.size() / t_size;

        if offset >= num_ts {
            panic!("Out of bounds {} * {} out of {}",
                   offset,
                   t_size,
                   self.size());
        }

        let slice: &mut [u8] = self.buf.borrow_mut();
        slice.as_mut_ptr().offset((offset * t_size) as isize) as *mut T
    }

    /// Get a reference of the given type to the offset in the buffer. The offset is relative to
    /// the size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ref<T>(&self, offset: usize) -> &T {
        &*self.get_ptr(offset)
    }

    /// Get a mutable reference of the given type to the offset in the buffer. The offset is
    /// relative to the size of `T`
    ///
    /// # Panics
    ///
    /// Panics if the offset is out of bounds
    pub unsafe fn get_ref_mut<T>(&mut self, offset: usize) -> &mut T {
        &mut *self.get_ptr_mut(offset)
    }
}
