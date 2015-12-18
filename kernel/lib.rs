//! The os1 kernel is written and compiled as a library. It is then
//! compiled and linked together with assembly files to produce a binary.
//! The binary is written to the hard disk, which is then loaded.
//!
//! The kernel is compiled without any `libstd`. Only `libcore` and
//! `liballoc` are used, since they provide core Rust functionality.

// To use unstable features of Rust, we need to have nightly rustc
#![feature(no_std,lang_items,alloc,core_str_ext,box_syntax,box_patterns,const_fn)]

// Compile without libstd
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

// use libcore
#[macro_use]
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
pub use self::memory::vmm_page_fault;

/// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // make sure interrupts are off
    unsafe { machine::cli(); }

    // print new line after "x"
    bootlog! ("\n");

    /////////////////////////////////////////////////////
    // Start initing stuff                             //
    /////////////////////////////////////////////////////

    // TODO: tss

    // init heap and vm
    memory::init();

    // init processes
    process::init();

    // init interupts
    interrupts::init(1000 /* hz */);

    /////////////////////////////////////////////////////
    // Done initing stuff                              //
    /////////////////////////////////////////////////////

    // yield to init process
    printf!("Everything inited! Here we go!\n");

    process::proc_yield(core::option::Option::None);

    // yield should never return to here
    panic! ("This should never happen!\n");
}
