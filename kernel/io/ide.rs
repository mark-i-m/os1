//! A module for accessing IDE block devices via PIO
//! TODO: Add DMA support for performance

use core::mem;

use super::block::*;
use machine::{inb, inl, outb, outl};
use process::{proc_yield, CURRENT_PROCESS};
use sync::StaticSemaphore;

/// The size of a sector
pub const SECTOR_SIZE: usize = 512;

/// The base I/O port for each controller
const PORTS: [u16; 2] = [0x1f0, 0x170];

/// An abstraction of an IDE block device
pub struct IDE {
    drive: u16,
    lock: StaticSemaphore,
}

/// IDE drive status
#[allow(dead_code)]
#[derive(PartialEq)]
enum IDEStatus {
    BUSY = 0x80,  // Busy
    READY = 0x40, // Read
    WTFLT = 0x20, // Drive write fault
    SCMPL = 0x10, // Drive seek complete
    DRQ = 0x08,   // Data request ready
    CORR = 0x04,  // Corrected data
    IDX = 0x02,   // Inlex
    ERR = 0x01,   // Error
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

    /// Get the status of the drive. This
    /// is a bit mask of status flags.
    #[inline]
    fn get_status(&self) -> u8 {
        unsafe { inb(self.port() + 7) }
    }

    /// Return true if the drive is busy
    #[inline]
    fn is_busy(&self) -> bool {
        self.get_status() & (IDEStatus::BUSY as u8) > 0
    }

    /// Return true if the drive is ready
    #[inline]
    fn is_ready(&self) -> bool {
        self.get_status() & (IDEStatus::READY as u8) > 0
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

    fn read_block(&mut self, block_num: usize, buffer: &mut BlockDataBuffer) {
        let base = self.port();
        let ch = self.channel();

        self.lock.down();

        // seek
        self.wait_for_drive();

        unsafe {
            outb(base + 2, 1); // block_num count
            outb(base + 3, ((block_num >> 0) & 0xFF) as u8); // bits 7 .. 0
            outb(base + 4, ((block_num >> 8) & 0xFF) as u8); // bits 15 .. 8
            outb(base + 5, ((block_num >> 16) & 0xFF) as u8); // bits 23 .. 16
            outb(
                base + 6,
                0xE0 | (ch << 4) as u8 | ((block_num >> 24) & 0xf) as u8,
            ); // bits 28 .. 24, send to primary master
            outb(base + 7, 0x20); // read with retry
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

    fn write_block(&mut self, block_num: usize, buffer: &BlockDataBuffer) {
        let base = self.port();
        let ch = self.channel();

        self.lock.down();

        // seek
        self.wait_for_drive();

        unsafe {
            outb(base + 2, 1); // block_num count
            outb(base + 3, ((block_num >> 0) & 0xFF) as u8); // bits 7 .. 0
            outb(base + 4, ((block_num >> 8) & 0xFF) as u8); // bits 15 .. 8
            outb(base + 5, ((block_num >> 16) & 0xFF) as u8); // bits 23 .. 16
            outb(
                base + 6,
                0xE0 | (ch << 4) as u8 | ((block_num >> 24) & 0xf) as u8,
            ); // bits 28 .. 24, send to primary master
            outb(base + 7, 0x30); // write with retry
        }

        // read
        self.wait_for_drive();

        let num_words = self.get_block_size() / mem::size_of::<u32>();
        for i in 0..num_words {
            unsafe {
                outl(base, *buffer.get_ref::<u32>(i));
            }
        }

        self.lock.up();
    }
}
