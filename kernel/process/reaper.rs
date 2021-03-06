//! A module for the reaper process, a process which frees the resources
//! of dead processes.

use alloc::boxed::Box;

use interrupts::no_interrupts;
use sync::StaticSemaphore;

use super::proc_table::PROCESS_TABLE;
use super::{ready_queue, Process, ProcessQueue};

/// The reaper blocks onto this semaphore until enough have died
static mut REAPER_SEMAPHORE: StaticSemaphore = StaticSemaphore::new(0);

/// The reaper queue. A queue of dead processes
static mut REAPER_QUEUE: ProcessQueue = ProcessQueue::new();

/// The reaper process routine:
/// If there are dead processes, pop them from the
/// REAPER_QUEUE and free their resources.
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
                let dead_proc =
                    no_interrupts(|| REAPER_QUEUE.pop_front().expect("Repear queue is empty"));

                PROCESS_TABLE.remove((*dead_proc).pid);

                // printf!("{:?} [Reaping]\n", *dead_proc);
                Box::from_raw(dead_proc);
                // let the box go out of scope to dealloc
            }
        }
    }
}

/// Add a dead process to the reaper queue.
/// NOTE: This should only be done for processes that will never run again
pub fn reaper_add(dead_proc: *mut Process) {
    no_interrupts(|| unsafe {
        REAPER_QUEUE.push_back(dead_proc);
        REAPER_SEMAPHORE.up();
    })
}

/// Create the reaper process and add it to the ready queue
pub fn init() {
    ready_queue::make_ready(Process::new("reaper", self::run));
}
