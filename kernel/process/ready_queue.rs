//Uncomment to debug
//use core::fmt::Display;
//use core::clone::Clone;

use alloc::boxed::Box;

use super::super::data_structures::{Queue, LazyGlobal};

use super::Process;

// ready q
static READY_QUEUE: LazyGlobal<Queue<Box<Process>>> = lazy_global!();

// init the ready q
pub fn init() {
    unsafe {
        READY_QUEUE.init(Queue::new());
    }
}

// get the ready q
pub fn ready_q<'a>() -> &'a mut Queue<Box<Process>> {
    unsafe {
        READY_QUEUE.get_mut()
    }
}

// Add the process to the Ready queue
pub fn make_ready(process: Box<Process>) {

    // TODO: lock here

    //TODO: get this right: (*process).set_state(State::READY);
    printf!("make ready {:?}\n", process);
    ready_q().push(process);

    // TODO: unlock here
}
