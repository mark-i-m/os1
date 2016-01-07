//! A module for accessing IDE block devices

use alloc::heap;

use core::mem;

use concurrency::StaticSemaphore;
use machine::{inb, inl, outb};
use process::{CURRENT_PROCESS, proc_yield};
use super::block::*;

/// The size of a sector
const SECTOR_SIZE: usize = 512;

/// The base I/O port for each controller
const PORTS: [u16; 2] = [0x1f0, 0x170];

/// An abstraction of an IDE block device
pub struct IDE {
    drive: u16,
    lock: StaticSemaphore,
}

/// IDE drive status
#[derive(PartialEq)]
enum IDEStatus {
    BUSY  = 0x80,   // Busy
    READY = 0x40,   // Read
    WTFLT = 0x20,   // Drive write fault
    SCMPL = 0x10,   // Drive seek complete
    DRQ   = 0x08,   // Data request ready
    CORR  = 0x04,   // Corrected data
    IDX   = 0x02,   // Inlex
    ERR   = 0x01,   // Error
}

/// A `BlockDataBuffer` for use with IDE devices
pub struct IDEBuf {
    buf: *mut u8,
    size: usize,
    offset: usize,
}

impl IDE {
    pub fn new(drive: u16) -> IDE {
        IDE {
            drive: drive,
            lock: StaticSemaphore::new(1),
        }
    }

    // The drive number encodes the controller in bit 1 and the channel in bit 0

    /// Get the controller for the drive
    #[inline]
    fn controller(&self) -> u16 {
        (self.drive >> 1) & 1
    }

    /// Get the channel for the drive
    #[inline]
    fn channel(&self) -> u16 {
        self.drive & 1
    }

    /// Get the port for the drive
    #[inline]
    fn port(&self) -> u16 {
        PORTS[self.controller() as usize]
    }

    /// Get the status of the drive
    #[inline]
    fn get_status(&self) -> IDEStatus {
        unsafe {
            match inb(self.port() + 7) {
                0x80 => IDEStatus::BUSY,
                0x40 => IDEStatus::READY,
                0x20 => IDEStatus::WTFLT,
                0x10 => IDEStatus::SCMPL,
                0x08 => IDEStatus::DRQ,
                0x04 => IDEStatus::CORR,
                0x02 => IDEStatus::IDX,
                0x01 => IDEStatus::ERR,
                _    => {
                    panic!("Invalid status");
                }
            }
        }
    }

    /// Return true if the drive is busy
    #[inline]
    fn is_busy(&self) -> bool {
        self.get_status() == IDEStatus::BUSY
    }

    /// Return true if the drive is ready
    #[inline]
    fn is_ready(&self) -> bool {
        self.get_status() == IDEStatus::READY
    }

    /// Wait for the drive to become ready
    #[inline]
    fn wait_for_drive(&self) {
        unsafe {
            while self.is_busy() {
                if !CURRENT_PROCESS.is_null() {
                    proc_yield(None);
                }
            }
            while !self.is_ready() {
                if !CURRENT_PROCESS.is_null() {
                    proc_yield(None);
                }
            }
        }
    }
}

impl BlockDevice for IDE {
    /// block size = sector size
    fn get_block_size(&self) -> usize {
        SECTOR_SIZE
    }

    fn read_block<B : BlockDataBuffer>(&mut self, block_num: usize, buffer: &mut B) {
        let base = self.port();
        let ch   = self.channel();

        self.lock.down();

        // seek
        self.wait_for_drive();

        unsafe {
            outb(base + 2, 1);			                        // block_num count
            outb(base + 3, ((block_num >> 0) & 0xFF) as u8);	// bits 7 .. 0
            outb(base + 4, ((block_num >> 8) & 0xFF) as u8);	// bits 15 .. 8
            outb(base + 5, ((block_num >> 16)& 0xFF) as u8);	// bits 23 .. 16
            outb(base + 6, 0xE0 | (ch << 4) as u8 | ((block_num >> 24) & 0xf) as u8);
            outb(base + 7, 0x20);		                        // read with retry
        }

        // read
        self.wait_for_drive();

        let num_words = self.get_block_size() / mem::size_of::<u32>();
        for i in 0..num_words {
            unsafe {
                *buffer.get_ref_mut::<u32>(i) = inl(base);
            }
        }

        self.lock.up();
    }
}

impl BlockDataBuffer for IDEBuf {
    fn new(size: usize) -> IDEBuf {
        unsafe {
            IDEBuf {
                buf: heap::allocate(size, 1),
                size: size,
                offset: 0,
            }
        }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn size(&self) -> usize {
        self.size
    }

    unsafe fn get_ptr<T>(&self, offset: usize) -> *mut T {
        let t_size = mem::size_of::<T>();
        let num_ts = self.size() / t_size;

        if offset >= num_ts {
            panic!("Out of bounds");
        }

        self.buf.offset((offset * t_size) as isize) as *mut T
    }
}

impl Drop for IDEBuf {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(self.buf, self.size, 1);
        }
    }
}
