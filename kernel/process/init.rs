use super::Process;
use super::ready_queue;

use alloc::boxed::Box;

use core::option::Option::None;

// The init process routine
pub fn run(this: &Process) -> usize {

    let p0 = Box::new(Process::new("p0", super::user::run));
    ready_queue::make_ready(p0);

    super::proc_yield(None);

    panic!("Yay!");
}
