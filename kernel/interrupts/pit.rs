//! A module for the programmable interrupt timer

use super::super::machine::pit_do_init;

/// Max frequency of the PIT
const FREQ: usize = 1193182;

/// The frequency of the PIT
static mut hz: usize = 0;

/// The number of jiffies passed since boot
#[allow(dead_code)]
pub static mut JIFFIES: usize = 0;

/// Initialize the PIT to the given frequency
pub fn init(pit_hz: usize) {
     let d = FREQ / pit_hz;

     if (d & 0xffff) != d {
         panic!("PIT init d={} doesn't fit in 16 bits", d);
     }

     unsafe {
         hz = FREQ / d;
         bootlog!("pit inited - requested {} hz, actual {} hz\n", pit_hz, hz);
         pit_do_init(d);
     }
}

/// Handle a PIT interrupt. Increments `JIFFIES`
pub fn handler() {
    unsafe {JIFFIES += 1;}
}

/// Calculate the number of seconds since boot
#[allow(dead_code)]
pub fn seconds() -> usize {
    unsafe {JIFFIES / hz}
}
