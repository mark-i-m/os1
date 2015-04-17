#![feature(no_std)]
#![feature(lang_items)]

#![no_std]

#![allow(dead_code)]

// minimal declarations of rust stuff
mod core;

// from rustboot on github:
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

//fn range(lo: i32, hi: i32) -> IntRange {
//    IntRange { cur: lo, max: hi }
//}

// These functions are invoked by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

fn clear_screen(background: u16) {
    let mut i = 0;
    while i < 80 * 25 {
        unsafe {
            *((0xb8000 + i * 2) as *mut u16) = background << 12;
        }
        i += 1;
    }
}

// this is the first rust to run in the kernel
// it is called by the kStart function in start.S
#[no_mangle]
#[no_stack_check]
pub fn kernel_main() {
    let color = Color::White as u16;
    clear_screen(color);
}
