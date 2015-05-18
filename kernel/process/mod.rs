// Implementing cooperative processes before turning on preemption

use alloc::boxed::{Box};
use core::option::Option::{self, Some, None};

pub use self::init::{Init};

pub mod init;

// The currently running process


// This is the API that all processes must implement
pub trait Process {
    fn new(name: &'static str) -> Self;
    fn run();
}

// This function initializes the process infrastructure
pub fn init() {
    printf!("In process init\n");

    // create Init process, but do not start it
    let test: Box<Process>;
    let init = Box::new(Init::new("Init"));

    // add to ready queue

    printf!("Created init process\n");
}

// Yield to the next process waiting
pub fn proc_yield() {

}
