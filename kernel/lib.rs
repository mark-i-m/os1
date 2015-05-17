#![feature(no_std,lang_items,core)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

// use libcore
#[macro_use]
extern crate core;
extern crate rlibc;

// kernel module imports
use window::{Window, Color};

// kernel module declarations
#[macro_use]
mod debug;
mod bare_bones;

mod machine;

mod vga;
mod window;

// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // clear the screen
    Window::clear_screen();

    // draw a test window
    draw_window();

    printf! ("Hi, {}!", "mark");
}

fn draw_window() {
    let mut w0 = Window::new(40, 10, (4, 10));

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    w0.set_cursor((1, 1));
    w0.set_bg_color(Color::LightGreen);
    w0.set_fg_color(Color::Red);
    w0.put_str("Hello World!");
}
