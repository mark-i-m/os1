use alloc::boxed::Box;

use core::option::Option::{self, Some, None};

use super::super::machine::{cli, sti, eflags};

use super::super::data_structures::{LazyGlobal};

use super::Process;

// current process
static CURRENT_PROCESS: LazyGlobal<Option<Box<Process>>> = lazy_global!();

///////////////////////////////////////////////////////////////
// DO NOT USE on/off here b/c there will be circular reference.
// Need to use cli/sti here, but be careful to leave interrupt
// flag as we found it.
///////////////////////////////////////////////////////////////

// Init the current process
pub fn init() {
    unsafe {
        CURRENT_PROCESS.init(None);
    }
}

// Get the current process
pub fn current<'a>() -> Option<&'a Box<Process>> {
    unsafe {
        // Are interrupts already disabled?
        let int_en = (eflags() & (1 << 9)) != 0;

        // disable interrupts
        cli();

        let ret = match *CURRENT_PROCESS.get() {
            Some(ref cp) => Some(cp),
            None => None,
        };

        // enable interrupts
        if int_en {
            sti();
        }

        ret
    }
}

// use for updating the current process
pub fn current_mut<'a>() -> Option<&'a mut Box<Process>> {
    unsafe {
        // Are interrupts already disabled?
        let int_en =(eflags() & (1 << 9)) != 0;

        // disable interrupts
        cli();

        let ret = match *CURRENT_PROCESS.get_mut() {
            Some(ref mut cp) => Some(cp),
            None => None,
        };

        // enable interrupts
        if int_en {
            sti();
        }

        ret
    }
}

// set the current process from a Box
pub fn set_current(process: Option<Box<Process>>) {
    unsafe {
        // Are interrupts already disabled?
        let int_en = (eflags() & (1 << 9)) != 0;

        // disable interrupts
        cli();

        *CURRENT_PROCESS.get_mut() = process;

        // enable interrupts
        if int_en {
            sti();
        }
    }
}
