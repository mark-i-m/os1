//! This module contains everything that has to do with memory
//!
//! 1. Kernel heap manager
//! 2. Physical memory allocator
//! 3. Virtual memory management

pub use self::virtmem::{vmm_page_fault, AddressSpace};

pub use self::tss::esp0;

pub mod rust_alloc;

mod heap;
mod tss;
mod physmem;
mod virtmem;
mod regionmap;

/// Initialize all memory subsystems
pub fn init(kheap_start: usize, kheap_size: usize) {
    tss::init();
    heap::init(kheap_start, kheap_size);
    physmem::init(kheap_start + kheap_size);
    virtmem::init(kheap_start + kheap_size + (4<<20));
}
