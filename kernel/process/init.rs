//! A module for the init process

#![allow(unused_variables)]

use vga::rectangle::Rectangle;
use super::Process;
use super::ready_queue;

/// The init process routine
pub fn run(this: &Process) -> usize {
    // clear the screen
    Rectangle::clear_screen();

    // start another process
    ready_queue::make_ready(Process::new("p0", super::user::run));

    0
}
