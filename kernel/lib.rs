#![feature(no_std,lang_items,core)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

#![allow(dead_code)]

// use libcore
#[macro_use]
extern crate core;
extern crate rlibc;

#[allow(unused_imports)]
use core::prelude::*;

#[allow(unused_imports)]
use bare_bones::*;

// kernel module imports
use vga::{Color, clear_screen};

// kernel module declarations
mod bare_bones;

mod vga;

// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // clear screen to LightBlue
    clear_screen(Color::LightBlue);
}
