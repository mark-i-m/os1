// Contains the reaper queue

use alloc::boxed::Box;

use super::{Process};

use super::super::data_structures::ProcessQueue;

use super::super::interrupts::{on, off};

// The reaper queue
static mut REAPER_QUEUE: ProcessQueue = ProcessQueue::new();

// The reaper process routine:,
// If there are dead processes, pop them from the
// REAPER_QUEUE and free the process's resources.
#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    loop {
        off();
        let dead_proc: *mut Process = unsafe {
            REAPER_QUEUE.pop_head()
        };
        on();

        if dead_proc.is_null() {
            break;
        } else {
            unsafe {
                printf!("{:?} [Reaping]\n", *dead_proc);
                Box::from_raw(dead_proc);
            }
            // let the box go out of scope to dealloc
        }
    }

    0
}

pub fn reaper_add(dead_proc: *mut Process) {
    off();
    unsafe {
        REAPER_QUEUE.push_tail(dead_proc);
    }
    on();
}
