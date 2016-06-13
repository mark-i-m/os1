//! A module for the init process

#![allow(unused_variables)]

use vga::rectangle::Rectangle;
use super::load::exec;
use super::Process;
use super::ready_queue;

use interrupts::{esp0, on, off};

/// The init process routine
pub fn run(this: &Process) -> usize {
    off();

    // clear the screen
    Rectangle::clear_screen();

    // start another process
    //ready_queue::make_ready(Process::new("p0", super::user::run));
    
    exec(1);

    panic!("Oh no!");

    0
}
