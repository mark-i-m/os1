#![allow(improper_ctypes, dead_code)]

use core::marker::{Sync, Send};

extern "C" {
    // add v to the value at ptr and return the old value
    pub fn fetchAndAdd32(ptr: &usize, v: usize) -> usize;
}

// An atomic 32-bit counter
#[derive(Copy, Clone)]
pub struct Atomic32 {
    pub i: usize,
}

impl Atomic32 {
    pub fn get_then_add(&self) -> usize {
        unsafe {fetchAndAdd32(&self.i, 1)}
    }
}

unsafe impl Sync for Atomic32 {}
unsafe impl Send for Atomic32 {}
