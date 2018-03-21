//! The os1 kernel is written and compiled as a library. It is then
//! compiled and linked together with assembly files to produce a binary.
//! The binary is written to the hard disk, which is then loaded.
//!
//! The kernel is compiled without any `libstd`. Only `libcore` and
//! `liballoc` are used, since they provide core Rust functionality.

// To use unstable features of Rust, we need to have nightly rustc
#![feature(lang_items,
           alloc,
           heap_api,
           global_allocator,
           allocator_api,
           box_syntax,
           box_patterns,
           const_fn,
           dropck_eyepatch,
           core_intrinsics,
          )]

// Compile without libstd
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "kernel"]

// use libcore
extern crate alloc;
extern crate rlibc;

// kernel module declarations
// debug must be first, since it defines macros the others need
#[macro_use]
mod debug;
mod bare_bones;

mod static_linked_list;

mod fs;
mod interrupts;
mod io;
mod machine;
mod memory;
mod process;
mod sync;
mod vga;

// exported functions -- to use in asm functions
pub use self::bare_bones::*;
pub use self::process::context::store_kcontext;
pub use self::process::{_proc_yield, syscall_handler};
pub use self::interrupts::pic::pic_irq;
pub use self::memory::vmm_page_fault;

/// The global allocator
#[global_allocator]
static ALLOCATOR: memory::KernelAllocator = memory::KernelAllocator;

/// This is the entry point to the kernel. It is the first rust code that runs.
#[no_mangle]
pub fn kernel_main() {
    // make sure interrupts are off
    unsafe {
        machine::cli();
    }

    // print new line after "x"
    bootlog!("\n");

    /////////////////////////////////////////////////////
    // Start initing stuff                             //
    /////////////////////////////////////////////////////

    // init tss, heap, and vm
    interrupts::tss_init();

    // make the kernel heap 3MiB starting at 1MiB.
    // make memory data structures take up the next 4MiB.
    memory::init(1 << 20, 3 << 20);

    // init processes
    process::init();

    // init interupts
    interrupts::init(1000 /* hz */);

    // filesystem
    fs::init(self::io::ide::IDE::new(3 /* hdd */));

    /////////////////////////////////////////////////////
    // Done initing stuff                              //
    /////////////////////////////////////////////////////

    // yield to init process
    printf!("Everything inited! Here we go!\n");

    process::proc_yield(None);

    // yield should never return to here
    panic!("This should never happen!\n");

}
