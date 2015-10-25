use super::{Process};
use super::ready_queue;

// idle process
pub static mut IDLE_PROCESS: *mut Process = 0 as *mut Process;


// Init the idle process
pub fn init() {
    unsafe {
        IDLE_PROCESS = Process::new("idle", self::run);
    }
}

// Just waste a quantum and hopefully there will be something to do
#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    loop{
        printf!("idle\n");
    }
}
