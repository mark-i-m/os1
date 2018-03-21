//! A module for the init process

#![allow(unused_variables)]

use vga::rectangle::Rectangle;
use super::load::exec;
use super::Process;
use super::ready_queue;

/// The init process routine
pub fn run(this: &Process) -> usize {
    // clear the screen
    Rectangle::clear_screen();

    exec(1);

    panic!("Oh no!");
}
