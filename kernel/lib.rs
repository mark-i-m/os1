#![feature(no_std,lang_items,core,alloc)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

// use libcore
#[macro_use]
extern crate core;
extern crate alloc;
extern crate rlibc;

// kernel module declarations
// debug must be first, since it defines macros the others need
#[macro_use]
mod debug;
mod bare_bones;
mod data_structures;

mod machine;

mod heap;
mod rust_alloc;

mod process;

mod vga;
mod window;

// kernel constans
pub const KHEAP_START: usize = (1 << 20); // 1M
pub const KHEAP_END: usize = (1 << 26); // 64M

// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // print new line after "x"
    printf! ("\n");

    // initialize the heap
    heap::init(KHEAP_START, KHEAP_END);
    printf! ("Heap inited\n");

    // initialize processes
    process::init();
    printf! ("Processes inited\n");

    // yield to init process
    process::proc_yield();

    printf! ("This should never happen!\n");
}
