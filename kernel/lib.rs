#![feature(no_std,lang_items,core)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

#![allow(dead_code)]

#[macro_use]
extern crate core;
extern crate rlibc;

use core::prelude::*;
use core::fmt;

enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Pink       = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    LightPink  = 13,
    Yellow     = 14,
    White      = 15,
}

fn clear_screen(background: u16) {
    for i in 0..(80 * 25) {
        unsafe {
            *((0xb8000 + i * 2) as *mut u16) = background << 12;
        }
    }
}

/**
 * This is the entry point to the kernel. It is the first rust code that runs.
 */
#[no_mangle]
pub fn kernel_main() {
    let color = Color::LightGreen as u16;
    clear_screen(color);
}


// For bare-bones rust
#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
pub extern fn rust_begin_unwind(args: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
//    use core::fmt::Write;
//    log!("\r\nPanic at {}:{}: ", file, line);
//    let _ = console::Console.write_fmt(args);
    loop {};
}
