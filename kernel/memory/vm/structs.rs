//! A module containing useful structs to abstract paging structures

use core::ops::{Index, IndexMut};

use core::intrinsics::transmute;

use super::super::super::interrupts::{on, off};

use super::super::physmem::Frame;

use super::VMM_ON;

use super::super::super::process::CURRENT_PROCESS;

/// A single entry in a page directory or table
/// ```
/// 31                 12  9       0
/// [                    000 flags  ]
/// ```
#[derive(Copy, Clone)]
#[repr(C,packed)]
pub struct PagingEntry {
    entry: usize,
}

/// An abstraction of page directories and page tables
#[repr(C,packed)]
pub struct VMTable {
    entries: [PagingEntry; 1024],
}

impl PagingEntry {
    /// Get a blank entry
    #[inline(always)]
    pub const fn new() -> PagingEntry {
        PagingEntry {
            entry: 0,
        }
    }

    // common flags

    /// Set the present bit.
    /// true = present; false = not present
    #[inline(always)]
    pub fn set_present(&mut self, value: bool) {
        self.set_flag(0, value);
    }

    /// Set the read/write bit.
    /// true = read/write, false = read-only
    #[inline(always)]
    pub fn set_read_write(&mut self, value: bool) {
        self.set_flag(1, value);
    }

    /// Set the privelege level bit.
    /// true = all, false = kernel mode only
    #[inline(always)]
    pub fn set_privelege_level(&mut self, value: bool) {
        self.set_flag(2, value);
    }

    /// Set caching bit.
    /// true = write-through, false = write-back
    #[inline(always)]
    pub fn set_caching(&mut self, value: bool) {
        self.set_flag(3, value);
    }

    // general ops

    /// Set the `index`-th flag to `value`.
    /// true = 1; false = 0
    #[inline(always)]
    pub fn set_flag(&mut self, index: u8, value: bool) {
        let mask = 1_usize << index;
        let entry = self.entry & !mask;
        self.entry = entry | if value {mask} else {0};
    }

    /// Set the upper 20 bits of the entry to the upper 20 bits of `address`
    #[inline(always)]
    pub fn set_address(&mut self, address: usize) {
        let addr = address & 0xFFFF_F000;
        let entry = self.entry & 0x0000_0FFF;

        self.entry = addr | entry;
    }

    // queries about the entry

    /// Return true if the `index`-th bit is set
    #[inline(always)]
    pub fn is_flag(&self, index: u8) -> bool {
        ((self.entry >> index) & 1) == 1
    }

    /// Return the upper 20-bits of the entry
    #[inline(always)]
    pub fn get_address(&self) -> usize {
        self.entry & 0xFFFF_F000
    }

    /// Free the frame pointed to if `dealloc` and mark this entry not present.
    pub fn free(&mut self, dealloc: bool) {
        off();
        if self.is_flag(0) {
            if dealloc {
                Frame::free(self.get_address() >> 12);
            }

            // clear the entry
            self.set_flag(0, false);
            self.set_flag(1, false);
            self.set_flag(2, false);
            self.set_flag(3, false);
            self.set_flag(4, false);
            self.set_flag(5, false);
            self.set_flag(6, false);
            self.set_flag(7, false);
            self.set_flag(8, false);
            self.set_flag(9, false);
            self.set_address(0);
        }
        on();
    }
}

impl VMTable {
    /// Returns a reference to a new `VMTable` and its physical address.
    /// The new table is kmapped if necessary, and is always cleared.
    pub fn new() -> (&'static mut VMTable, usize) {
        // allocate a frame
        let paddr = Frame::alloc();

        let mut table: &mut VMTable = unsafe {
            // if VM is on, kmap this frame
            // otherwise use the paddr
            let frame = if VMM_ON {
                if !CURRENT_PROCESS.is_null() {
                    (*CURRENT_PROCESS).addr_space.kmap(paddr)
                } else {
                    panic!("VMM_ON with no CURRENT_PROCESS!");
                }
            } else {
                &mut *(paddr as *mut Frame)
            };

            transmute(frame)
        };

        // clear the frame
        for i in 0..1024 {
            table[i] = PagingEntry::new();
        }

        (table, paddr)
    }
}

/// Make `VMTable` indexable
impl Index<usize> for VMTable {
    type Output = PagingEntry;

    fn index<'a>(&'a self, index: usize) -> &'a PagingEntry {
        unsafe { transmute::<&usize, &PagingEntry>(
                &transmute::<&VMTable, &Frame>(self)[index]) }
    }
}

/// Make `VMTable` indexable
impl IndexMut<usize> for VMTable {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut PagingEntry {
        unsafe { transmute::<&mut usize, &mut PagingEntry>(
                &mut transmute::<&mut VMTable, &mut Frame>(self)[index]) }
    }
}
