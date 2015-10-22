// The programmable interrupt timer

use super::super::machine::pit_do_init;

const FREQ: usize = 1193182;
static mut hz: usize = 0;

#[allow(dead_code)]
static mut jiffies: usize = 0;

pub fn init(pit_hz: usize) {
     let d = FREQ / pit_hz;

     if (d & 0xffff) != d {
         panic!("PIT init d={} doesn't fit in 16 bits", d);
     }

     unsafe {
         hz = FREQ / d;
         printf!("PIT inited: requested {} hz, actual {} hz\n", pit_hz, hz);
         pit_do_init(d);
     }
}

pub fn handler() {
    unsafe {jiffies += 1;}
}

#[allow(dead_code)]
pub fn seconds() -> usize {
    unsafe {jiffies / hz}
}
