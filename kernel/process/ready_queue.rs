// This file contains functions for push/pop to the ready queue

use super::super::interrupts::{on, off};
use super::{Process, State};
use super::super::data_structures::{ProcessQueue};

// points to the next process to run
pub static mut READY_QUEUE: ProcessQueue = ProcessQueue {
    head: 0 as *mut Process,
    tail: 0 as *mut Process
};
//pub static mut READY_QUEUE: ProcessQueue = ProcessQueue::new();

// init the ready q
pub fn init() {
}

// Add the process to the Ready queue
pub fn make_ready(process: *mut Process) {
    // disable interrupts
    off();

    unsafe {
        //bootlog!("Readying {:?}\n", *process);
        (*process).set_state(State::READY);
        READY_QUEUE.push_tail(process);
    }

    // enable interrupts
    on();
}

// Deque and return the next ready process
// NOTE: returns null if there are no ready processes
pub fn get_next() -> *mut Process {
    unsafe {
        // disable interrupts
        off();

        let ret = READY_QUEUE.pop_head();

        // enable interrupts
        on();

        ret
    }
}
