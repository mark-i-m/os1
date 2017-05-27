//! A module for the ready queue

use interrupts::no_preempt;
use super::{Process, State, ProcessQueue};

/// The ready queue. A queue containing all processes that are ready to be scheduled.
pub static mut READY_QUEUE: ProcessQueue = ProcessQueue::new();

/// Add the process to the ready queue
pub fn make_ready(process: *mut Process) {
    no_preempt(|| unsafe {
                   // bootlog!("{:?} [Ready]\n", *process);
                   (*process).set_state(State::READY);
                   READY_QUEUE.push_back(process);
               })
}

/// Unqueue and return the next ready process.
/// NOTE: returns null if there are no ready processes
pub fn get_next() -> *mut Process {
    unsafe {
        let ret = no_preempt(|| READY_QUEUE.pop_front());

        if let Some(next) = ret {
            next
        } else {
            0 as *mut Process
        }
    }
}
