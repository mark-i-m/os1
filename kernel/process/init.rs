#![allow(unused_variables)]

use super::Process;
use super::ready_queue;

//use alloc::boxed::Box;

use core::option::Option::None;

// The init process routine
pub fn run(this: &Process) -> usize {
    printf!("init about to yield\n");

    super::proc_yield(None);
    super::proc_yield(None);
    super::proc_yield(None);

    ready_queue::make_ready(Process::new("p0", super::user::run));
    super::proc_yield(None);

    super::proc_yield(None);
    super::proc_yield(None);
    super::proc_yield(None);
    super::proc_yield(None);
    super::proc_yield(None);

    0xDEADBEEF
}
