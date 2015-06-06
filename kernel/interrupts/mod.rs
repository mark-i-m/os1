// This module contains everything needed for interrupts

mod idt;

pub mod pit;
pub mod pic;

pub fn init(pit_hz: usize) {
    pic::init();
    pit::init(pit_hz);
}
