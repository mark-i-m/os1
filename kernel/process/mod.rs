// Implementing cooperative processes before turning on preemption

use core::marker::{Sized};
use alloc::boxed;
use alloc::boxed::{into_raw, Box};

pub use self::init::{Init};

pub mod init;

// The currently running process
// access through process::current(), and process::set_current()

use self::CurrentProc::*;

#[allow(dead_code)]
enum CurrentProc {
    Ptr(*mut Process),
    Null,
}

#[allow(dead_code)]
static mut current_proc: CurrentProc = Null;

// This is the API that all processes must implement
//
// note: if methods does not take &self, then Process is not object-safe
pub trait Process {
    fn new(name: &'static str) -> Self
        where Self : Sized;
    fn run(&self) -> usize;
}

// This function initializes the process infrastructure
pub fn init() {
    printf!("In process init\n");

    // create Init process, but do not start it
    let init: Box<Process> = Box::new(Init::new("Init"));

    // add to ready queue
    set_current(init);

    printf!("Created init process\n");
}

#[allow(dead_code)]
pub fn current() -> Box<Process> {
    unsafe {
        match current_proc {
            Ptr(p) => {
                Box::from_raw(p)
            }
            Null   => {
                panic!("No current process");
            }
        }
    }
}

#[allow(dead_code)]
pub fn set_current(process: Box<Process>) {
    unsafe {
        current_proc = Ptr(boxed::into_raw(process));
    }
}

// Yield to the next process waiting
pub fn proc_yield() {

}
