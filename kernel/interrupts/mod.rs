//! This module contains everything needed for interrupts

pub use self::process::{on, off};

pub use self::idt::add_trap_handler;

mod idt;
mod process;

pub mod pit;
pub mod pic;

/// Initialize interrupts. Set the PIT frequency to `pit_hz`
pub fn init(pit_hz: usize) {
    pic::init();
    pit::init(pit_hz);
}
