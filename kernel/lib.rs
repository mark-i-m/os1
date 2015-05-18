#![feature(no_std,lang_items,core,alloc)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

// use libcore
#[macro_use]
extern crate core;
extern crate alloc;
extern crate rlibc;

use alloc::boxed::{Box};

// kernel module imports
use window::{Window, Color};

use heap::{init};

// kernel module declarations
#[macro_use]
mod debug; // debug must be first, since it defines macros the others need
mod bare_bones;

mod machine;

mod vga;
mod window;

mod heap;
mod rust_alloc;

// kernel constans
pub const KHEAP_START: usize = (1 << 20); // 1M
pub const KHEAP_END: usize = (1 << 26); // 64M

// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // print new line after "x"
    printf! ("\n");

    // clear the screen
    Window::clear_screen();

    // draw a test window to let the world know we're alive
    draw_window();

    // initialize the heap
    printf! ("Going to init heap\n");
    heap::init(KHEAP_START, KHEAP_END);
    printf! ("Heap inited\n");

    printf! ("Testing malloc\n");
    test_malloc();
    printf! ("Done!");
}

// Draw a test window
fn draw_window() {
    let mut w0 = Window::new(40, 10, (4, 10));

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    w0.set_cursor((0, 0));
    w0.set_bg_color(Color::LightGreen);
    w0.set_fg_color(Color::Red);
    w0.put_str("Hello World!");
}

fn test_malloc() {
    let x = Box::new(5 as i32);

    test_malloc2(x);
}

fn test_malloc2(x: Box<i32>) {
    if *x != 5 {
        panic!("AAAH!");
    }

    let y = Box::new(6 as usize);
}
