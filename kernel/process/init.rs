#![allow(unused_variables)]

use super::Process;
use super::ready_queue;

use alloc::boxed::Box;

use core::option::Option::None;

// The init process routine
pub fn run(this: &Process) -> usize {
    printf!("init about to yield\n");

    unsafe{super::proc_yield(None);}

    printf!("back from yield!\n");

    let p0 = Process::new("p0", super::user::run);

    panic!("Created p0\n");

    ready_queue::make_ready(p0);

    unsafe{super::proc_yield(None);}

    0xDEADBEEF
}
