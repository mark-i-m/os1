use alloc::boxed::Box;

use core::clone::Clone;
use core::option::Option::{self, Some, None};

use super::super::interrupts::{on, off};

use super::super::data_structures::{LazyGlobal};

use super::Process;

// current process
static CURRENT_PROCESS: LazyGlobal<Option<Box<Process>>> = lazy_global!();

// Init the current process
pub fn init() {
    unsafe {
        CURRENT_PROCESS.init(None);
    }
}

// Get the current process
// Must use a Box because size of Process is unknown at compile time
pub fn current() -> Option<Box<Process>> {
    // disable interrupts
    //off();

    let ret = unsafe {
        (*CURRENT_PROCESS.get()).clone()
    };

    // enable interrupts
    //on();

    ret
}

// use for updating the current process
pub fn current_mut<'a>() -> Option<&'a mut Box<Process>> {
    // disable interrupts
    //off();

    let ret = unsafe {
        match *CURRENT_PROCESS.get_mut() {
            Some(ref mut cp) => Some(cp),
            None => None,
        }
    };

    // enable interrupts
    //on();

    ret
}

// set the current process from a Box
pub fn set_current(process: Option<Box<Process>>) {
    // disable interrupts
    //off();

    unsafe {
        *CURRENT_PROCESS.get_mut() = process;
    }

    // enable interrupts
    //on();
}
