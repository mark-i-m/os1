use alloc::boxed;
use alloc::boxed::Box;

use core::option::Option::{self, Some, None};

use super::super::data_structures::Queue;

use super::Process;

// current process
static mut current_process: Option<*mut Process> = None;

// Get the current process
// Must use a Box because size of Process is unknown at compile time
pub fn current() -> Option<Box<Process>> {
    // TODO: lock here
    unsafe {
        match current_process {
            Some(p) => { Some(Box::from_raw(p)) }
            None => { None }
        }
    }
    // TODO: lock here
}

// set the current process from a Box
pub fn set_current(process: Option<Box<Process>>) {
    // TODO: lock here

    unsafe {
        match process {
            Some(p) => {
                current_process = Some(boxed::into_raw(p));
            }
            None => {
                current_process = None;
            }
        }
    }

    // TODO: unlock here
}
