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
    // make sure interrupts are off
    unsafe { machine::cli(); }

    // print new line after "x"
    bootlog! ("\n");

    // TODO: tss

    // initialize stuff
    memory::init(KHEAP_START, KHEAP_END);

    process::init(); // this creates Process #0: init

    /////////////////////////////////////////////////////
    // DO NOT USE interrupts::on()/off() BEFORE HERE   //
    // DO NOT USE printf!() BEFORE HERE; USE bootlog!()//
    /////////////////////////////////////////////////////

    interrupts::init(100 /* hz */);

    // yield to init process
    printf!("Everything inited! Here we go!\n");
    process::proc_yield(core::option::Option::None);

    // yield should never return to here
    panic! ("This should never happen!\n");
}
