//! A module for the idle process, a process that runs when there is
//! nothing else left to do.

use core::option::Option::{None};

use super::{Process, proc_yield};

/// The idle process
pub static mut IDLE_PROCESS: *mut Process = 0 as *mut Process;

/// Create the idle process but do not start it until we need it
pub fn init() {
    unsafe {
        IDLE_PROCESS = Process::new("idle", self::run);
    }
}

/// Just waste a quantum and hopefully there will be something to do
#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    loop{
        super::proc_yield(None);
        panic!("idle");
    }
}
