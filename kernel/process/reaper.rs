// Contains the reaper queue

use alloc::boxed::Box;

use super::{Process, ready_queue};

use super::super::data_structures::ProcessQueue;
use super::super::data_structures::concurrency::StaticSemaphore;

use super::super::interrupts::{on, off};

// The reaper blocks onto this semaphore until enough have died
static mut REAPER_SEMAPHORE: StaticSemaphore = StaticSemaphore::new(0);

// The reaper queue
static mut REAPER_QUEUE: ProcessQueue = ProcessQueue::new();

// The reaper process routine:,
// If there are dead processes, pop them from the
// REAPER_QUEUE and free the process's resources.
#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    unsafe {
        loop {
            // wait for 10 dead processes
            for _ in 0..10 {
                REAPER_SEMAPHORE.down();
            }

            // reap the 10 processes
            for _ in 0..10 {
                off();
                let dead_proc: *mut Process = REAPER_QUEUE.pop_head();
                on();

                printf!("{:?} [Reaping]\n", *dead_proc);
                Box::from_raw(dead_proc);
                // let the box go out of scope to dealloc
            }
        }
    }

    0
}

pub fn reaper_add(dead_proc: *mut Process) {
    off();
    unsafe {
        REAPER_QUEUE.push_tail(dead_proc);
        REAPER_SEMAPHORE.up();
    }
    on();
}

pub fn init() {
    ready_queue::make_ready(Process::new("reaper", self::run));
}
