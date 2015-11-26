// This module contains everything needed for interrupts

pub use self::process::{on, off};

pub use self::idt::add_trap_handler;

mod idt;
mod process;

pub mod pit;
pub mod pic;

pub fn init(pit_hz: usize) {
    pic::init();
    pit::init(pit_hz);
}
