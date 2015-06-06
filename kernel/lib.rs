#![feature(no_std,lang_items,core,alloc,box_syntax,box_patterns,debug_builders)]
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

#[macro_use]
mod data_structures;
mod machine;
mod memory;
mod process;
mod vga;
mod interrupts;

// exported functions -- to use in asm functions
pub use self::process::context::store_kcontext;
pub use self::process::_proc_yield;
pub use self::interrupts::pic::pic_irq;

// kernel constans
pub const KHEAP_START: usize = (1 << 20); // 1M
pub const KHEAP_END: usize = (1 << 26); // 64M

// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    use core::option::Option::None;

    // print new line after "x"
    printf! ("\n");

    // TODO: tss
    // TODO: idt

    // initialize the heap
    memory::init(KHEAP_START, KHEAP_END);
    printf! ("Heap inited\n");

    // initialize processes
    process::init();
    printf! ("Processes inited\n");

    interrupts::init(1000); // pit hz

    // yield to init process
    unsafe{process::proc_yield(None);}

    panic! ("This should never happen!\n");
}
