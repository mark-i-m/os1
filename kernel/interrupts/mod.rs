//! This module contains everything needed for interrupts

pub use self::idt::add_trap_handler;
pub use self::process::{on, off};
pub use self::tss::esp0;
pub use self::tss::init as tss_init;

pub mod pic;
pub mod pit;

mod idt;
mod process;
mod tss;

/// Initialize interrupts. Set the PIT frequency to `pit_hz`
pub fn init(pit_hz: usize) {
    pic::init();
    pit::init(pit_hz);
}
