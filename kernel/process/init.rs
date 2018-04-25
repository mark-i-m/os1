//! A module for the init process

#![allow(unused_variables)]

use super::load::exec;
use super::ready_queue;
use super::Process;
use vga::rectangle::Rectangle;

/// The init process routine
pub fn run(this: &Process) -> usize {
    // clear the screen
    Rectangle::clear_screen();

    exec(1);

    panic!("Oh no!");
}
