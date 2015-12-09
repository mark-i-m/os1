// Physical memory management

use core::ops::{Index, IndexMut};

use super::super::interrupts::{add_trap_handler, on, off};

use super::super::machine::page_fault_handler;

use super::regionmap::RegionMap;

// array of frame info
static mut FRAME_INFO: *mut FrameInfoSection = 0 as *mut FrameInfoSection;

// index of the first free frame in a LIFO list
// 0 => empty list
static mut FREE_FRAMES: usize = 0;

// A physical memory frame
#[repr(C, packed)]
pub struct Frame {
    mem: [usize; 1024],
}

// should take up 4MiB on a 32-bit machine
// there is one frame info for each frame, associated
// by frame index
#[repr(C, packed)]
pub struct FrameInfoSection {
    arr: [FrameInfo; 1<<20],
}

// Frame info
#[repr(C, packed)]
pub struct FrameInfo {
    info: usize,
}

impl Frame {
    // allocate a frame and return its physical address
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

    // free this frame
    pub fn free(index: usize) {
        let all_frames = unsafe {&mut *FRAME_INFO};

        off();

        all_frames[index].free();

        on();
    }
}

impl Index<usize> for Frame {
    type Output = usize;

    fn index<'a>(&'a self, index: usize) -> &'a usize {
        &self.mem[index]
    }
}

impl IndexMut<usize> for Frame {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut usize {
        &mut self.mem[index]
    }
}

impl Index<usize> for FrameInfoSection {
    type Output = FrameInfo;

    fn index<'a>(&'a self, index: usize) -> &'a FrameInfo {
        &self.arr[index]
    }
}

impl IndexMut<usize> for FrameInfoSection {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut FrameInfo {
        &mut self.arr[index]
    }
}

impl FrameInfo {
    // allocate the frame referred to by this FrameInfo
    // NOTE this should only be called on the first frame in the free list
    pub fn alloc(&mut self) {
        off();

        // Remove from list
        unsafe {
            FREE_FRAMES = self.get_next_free();
        }

        on();

        // mark not free
        self.set_free(false);
    }

    // free the frame referred to by this FrameInfo
    pub fn free(&mut self) {
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

    pub fn get_index(&self) -> usize {
        let first = unsafe { FRAME_INFO as *const FrameInfoSection as usize };
        let addr = self as *const FrameInfo as usize;
        (addr - first) / 4
    }

    fn set_free(&mut self, free: bool) {
        // TODO
    }

    fn get_next_free(&self) -> usize {
        0
            //TODO
    }

    fn set_next_free(&mut self, next: usize) {
        // TODO
    }
}

// init phys mem frames using the rest of physical memory
pub fn init(start: usize) {
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
    let mut mem = RegionMap::new((8<<20) as *mut Frame);
    let mut num_frames = 0;

    // add all available frames to free list
    while let Some((base, num)) = mem.next_avail() {
        for i in base..(base+num) {
            all_frames[i].free();
        }
        num_frames += num;
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    bootlog!("phys mem inited: {} frames\n", num_frames);
}
