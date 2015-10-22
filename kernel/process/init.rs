#![allow(unused_variables)]

use super::Process;
use super::ready_queue;

// The init process routine
pub fn run(this: &Process) -> usize {
    // start another process
    ready_queue::make_ready(Process::new("p0", super::user::run));

    // TODO: join instead of loop forever
    loop{}
}
