use alloc::boxed;
use alloc::boxed::Box;

use core::option::Option::{self, Some, None};

use super::super::data_structures::Queue;

use super::Process;

// ready q
static mut ready_queue: Option<*mut Queue<Box<Process>>> = None;

// init the ready q
pub fn init() {
    unsafe {
        ready_queue = Some(boxed::into_raw(Box::new(Queue::new())));
    }
}

// get the ready q
pub fn ready_q() -> Box<Queue<Box<Process>>> {
    unsafe {
        match ready_queue {
            Some(q) => { Box::from_raw(q) }
            None => { panic!("Unitialized ready queue") }
        }
    }
}

// Add the process to the Ready queue
pub fn make_ready(process: Box<Process>) {

    // TODO: lock here

    //TODO: get this right: (*process).set_state(State::READY);
    (*ready_q()).push(process);

    // TODO: unlock here
}
