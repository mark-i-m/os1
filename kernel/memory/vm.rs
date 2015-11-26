// Module for virtual memory

use super::super::interrupts::add_trap_handler;

use super::super::machine::{vmm_on, invlpg, page_fault_handler};

// beginning and end of available frames
// NOTE: cannot use *mut [Frame] because this has size of 8B for some reason
static mut PHYS_MEM_BEGIN: *mut Frame = 0 as *mut Frame;
static mut PHYS_MEM_END: *mut Frame = 0 as *mut Frame;

// stack of free frames as a linked list
static mut FREE_FRAMES: *mut Frame = 0 as *mut Frame;

#[repr(C, packed)]
struct Frame {
    mem: [usize; 1024],
}

impl Frame {
    fn alloc () -> &'static Frame {
        unsafe {
            if FREE_FRAMES.is_null() {
                // TODO: page out
                panic!("Out of physical memory");
            }

            // Remove from free list
            let free = &*FREE_FRAMES;
            FREE_FRAMES = (*FREE_FRAMES).get_next();

            // Return the free frame
            free
        }
    }

    fn free (&mut self) {
        unsafe {
            // push to front of free stack/list
            let next = FREE_FRAMES;
            self.set_next(next);
            FREE_FRAMES = self as *mut Frame;
        }
    }

    unsafe fn set_next(&mut self, next: *mut Frame) {
        self.mem[0] = next as usize;
    }

    unsafe fn get_next(&self) -> *mut Frame {
        self.mem[0] as *mut Frame
    }
}

// init phys mem frames
pub fn init(mut start: usize, mut end: usize) {
    // round start up to nearest page boundary
    start = if start % 0x1000 == 0 {
        start
    } else {
        start - (start % 0x1000) + 0x1000
    };
    unsafe { PHYS_MEM_BEGIN = start as *mut Frame; }

    // round end down to nearest page boundary
    end = end & !0x1000;
    unsafe { PHYS_MEM_END = end as *mut Frame; }

    // add all frames to free list
    // in reverse order because it is a stack
    // and because I like them being used in order
    unsafe {
        let mut frame = PHYS_MEM_END.offset(-1);
        while frame >= PHYS_MEM_BEGIN {
            (*frame).free();
            frame = frame.offset(1);
        }
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    bootlog!("Phys Mem inited\n");
}

// handle for vmm for asm
#[no_mangle]
pub extern "C" fn vmm_page_fault(/*context: *mut KContext,*/ fault_addr: *mut usize) {

}
