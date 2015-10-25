// Contains the reaper queue

use core::option::Option::{None};

use super::{Process};

use super::super::data_structures::ProcessQueue;

use super::super::interrupts::{on, off};

// The reaper queue
static mut REAPER_QUEUE: ProcessQueue = ProcessQueue {
    head: 0 as *mut Process,
    tail: 0 as *mut Process
};

// The reaper process routine: during our quantum, pop from the
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
            unsafe {
                super::proc_yield(None);
            }
        } else {
            unsafe {
                printf!("Reaping {:?}\n", *dead_proc);
            }
            super::Process::destroy(dead_proc);
        }
    }
}

pub fn reaper_add(dead_proc: *mut Process) {
    off();
    unsafe {
        REAPER_QUEUE.push_tail(dead_proc);
    }
    on();
}
