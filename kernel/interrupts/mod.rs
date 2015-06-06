// This module contains everything needed for interrupts

pub use self::pic::pic_irq;

mod idt;

mod pit;
mod pic;

pub fn init() {

}
