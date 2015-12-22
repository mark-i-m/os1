//! Physical memory management

use core::ops::{Index, IndexMut};

use super::super::interrupts::{on, off};

use super::regionmap::RegionMap;

/// Array of `FrameInfo`. This is a pointer to the region of memory used
/// to hold physical memory allocation metadata.
static mut FRAME_INFO: *mut FrameInfoSection = 0 as *mut FrameInfoSection;

/// The index of the first free frame in a LIFO list.
/// 0 => empty list
static mut FREE_FRAMES: usize = 0;

/// A physical memory frame
#[repr(C, packed)]
pub struct Frame {
    mem: [usize; 1024],
}

/// Should take up 4MiB on a 32-bit machine.
/// There is one frame info for each frame, associated by frame index
#[repr(C, packed)]
pub struct FrameInfoSection {
    arr: [FrameInfo; 1<<20],
}

/// Frame info
/// ```
/// When free
/// 31                 20           0
/// [nnnnnnnnnnnnnnnnnnnn           1]
///  ^-- next free index            ^-- free bit
///
/// When allocated
/// 31                           30 0
/// [ssssssssssssssssssssssssssssss 0]
///  ^-- ptr to shared frame info   ^-- free bit
/// ```
#[repr(C, packed)]
pub struct FrameInfo {
    info: usize,
}

impl Frame {
    /// Allocate a frame and return its physical address
    pub fn alloc() -> usize {
        let all_frames = unsafe {&mut *FRAME_INFO};

        off();

        // get a frame
        let free = unsafe{FREE_FRAMES};

        if free == 0 {
            // TODO: page out
            panic!("Out of physical memory");
        }

        all_frames[free].alloc();

        on();

        free << 12
    }

    /// Free the frame with the given index
    pub fn free(index: usize) {
        let all_frames = unsafe {&mut *FRAME_INFO};

        off();
        all_frames[index].free();
        on();
    }
}

/// Make the words of a frame indexable
impl Index<usize> for Frame {
    type Output = usize;

    fn index<'a>(&'a self, index: usize) -> &'a usize {
        &self.mem[index]
    }
}

/// Make the words of a frame indexable
impl IndexMut<usize> for Frame {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut usize {
        &mut self.mem[index]
    }
}

/// Make the `FrameInfoSection` indexable
impl Index<usize> for FrameInfoSection {
    type Output = FrameInfo;

    fn index<'a>(&'a self, index: usize) -> &'a FrameInfo {
        &self.arr[index]
    }
}

/// Make the `FrameInfoSection` indexable
impl IndexMut<usize> for FrameInfoSection {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut FrameInfo {
        &mut self.arr[index]
    }
}

impl FrameInfo {
    /// Allocate the frame referred to by this `FrameInfo`.
    /// NOTE: this should only be called on the first frame in the free list
    pub fn alloc(&mut self) {
        if self.get_index() != unsafe {FREE_FRAMES} {
            panic!("Attempt to alloc middle free frame {}", self.get_index());
        }

        off();

        // Remove from list
        unsafe {
            FREE_FRAMES = self.get_next_free();
        }

        on();

        // mark not free
        self.set_free(false);
    }

    /// Free the frame referred to by this FrameInfo
    pub fn free(&mut self) {
        // TODO: handle case of shared pages

        // mark free
        self.set_free(true);

        // add to free list
        off();

        unsafe {
            self.set_next_free(FREE_FRAMES);
            FREE_FRAMES = self.get_index();
        }

        on();
    }

    /// Get the index of the `Frame`
    pub fn get_index(&self) -> usize {
        let first = unsafe { FRAME_INFO as *const FrameInfoSection as usize };
        let addr = self as *const FrameInfo as usize;
        (addr - first) / 4
    }

    /// Set the free bit of this `FrameInfo` to 1 if `free` is true; else 0
    fn set_free(&mut self, free: bool) {
        let base = self.info & !1;
        self.info = base | if free { 1 } else { 0 }
    }

    /// Get the index of the next free frame
    fn get_next_free(&self) -> usize {
        self.info >> 12
    }

    /// Set the index of the next free frame
    fn set_next_free(&mut self, next: usize) {
        let base = self.info & 0xFFF;
        self.info = (next << 12) | base;
    }
}

/// Initialize physical memory frames using the rest of physical memory.
/// This function detects all available physical memory.
pub fn init(start: usize) {
    if start % 0x400000 != 0 {
        panic!("phys mem start must be 4MiB aligned!");
    }

    unsafe {
        // round start up to nearest page boundary
        FRAME_INFO = if start % 0x1000 == 0 {
            start
        } else {
            start - (start % 0x1000) + 0x1000
        } as *mut FrameInfoSection;
    }

    let all_frames = unsafe {&mut *FRAME_INFO};

    // Detect available memory
    let mem = RegionMap::new((8<<20) as *mut Frame);
    let mut num_frames = 0;

    // add all available frames to free list
    for (base, num) in mem {
        for i in base..(base+num) {
            all_frames[i].free();
        }
        num_frames += num;
    }

    bootlog!("phys mem inited - metadata @ 0x{:X}, {} frames\n", start, num_frames);
}
