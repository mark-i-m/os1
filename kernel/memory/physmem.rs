// Physical memory management

use core::ops::{Index, IndexMut};

use super::super::interrupts::{add_trap_handler, on, off};

use super::super::machine::page_fault_handler;

use super::regionmap::RegionMap;

// beginning of available frames
pub static mut PHYS_MEM_BEGIN: *mut Frame = 0 as *mut Frame;

// stack of free frames as a linked list
static mut FREE_FRAMES: *mut Frame = 0 as *mut Frame;

// A physical memory frame
#[repr(C, packed)]
pub struct Frame {
    mem: [usize; 1024],
}

impl Frame {
    // allocate a frame and return a reference to it
    pub fn new () -> &'static mut Frame {
        unsafe {
            off();

            if FREE_FRAMES.is_null() {
                // TODO: page out
                panic!("Out of physical memory");
            }

            // Remove from free list
            let free = &mut*FREE_FRAMES;
            FREE_FRAMES = (*FREE_FRAMES).get_next();

            // zero out memory
            let mut i = 0;
            while i < 1024 {
                free[i] = 0;
                i += 1;
            }

            on();

            // Return the free frame
            free
        }
    }

    // free this frame
    pub fn free (&mut self) {
        off();
        unsafe {
            // push to front of free stack/list
            let next = FREE_FRAMES;
            self.set_next(next);
            FREE_FRAMES = self as *mut Frame;
        }
        on();
    }

    unsafe fn set_next(&mut self, next: *mut Frame) {
        self.mem[0] = next as usize;
    }

    unsafe fn get_next(&self) -> *mut Frame {
        self.mem[0] as *mut Frame
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

// init phys mem frames using the rest of physical memory
pub fn init(start: usize) {
    unsafe {
        // round start up to nearest page boundary
        PHYS_MEM_BEGIN = if start % 0x1000 == 0 {
            start
        } else {
            start - (start % 0x1000) + 0x1000
        } as *mut Frame;
    }

    // Detect available memory
    let mut mem = RegionMap::new(PHYS_MEM_BEGIN);

    // add all available frames to free list
    let mut num_frames = 0;
    while let Some((base, num)) = mem.next_avail() {
        for i in 0..num {
            base[i].free();
        }
        num_frames += num;
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    unsafe{
        bootlog!("phys mem start addr 0x{:X}, {} frames\n",
                PHYS_MEM_BEGIN as usize, num_frames);
    }
}
