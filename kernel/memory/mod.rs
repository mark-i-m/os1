//! This module contains everything that has to do with memory
//!
//! 1. Kernel heap manager
//! 2. Physical memory allocator
//! 3. Virtual memory management

pub use self::virtmem::{vmm_page_fault, AddressSpace};

pub mod rust_alloc;

mod heap;
mod physmem;
mod virtmem;
mod regionmap;

/// Initialize all memory subsystems
pub fn init() {
    // make the kernel heap 3MiB starting at 1MiB.
    // make memory data structures take up the next 4MiB.
    heap::init(1<<20, 3<<20);
    physmem::init(4<<20);
    virtmem::init();
}
