//! This module contains everything needed for interrupts

pub use self::idt::add_trap_handler;
pub use self::tss::esp0;
pub use self::tss::init as tss_init;

use self::process::{on, off};

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

/// Runs closure `f` without preemption
#[inline(always)]
pub fn no_preempt<T, F: FnOnce() -> T>(f: F) -> T {
    process::off();

    let result = f();

    process::on();

    result
}
