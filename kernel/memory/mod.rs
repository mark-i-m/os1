//! This module contains everything that has to do with memory
//!
//! 1. Kernel heap manager
//! 2. Physical memory allocator
//! 3. Virtual memory management

pub use self::heap::KernelAllocator;
pub use self::vm::{vmm_page_fault, AddressSpace};

mod heap;
mod physmem;
mod regionmap;
mod vm;

/// Initialize all memory subsystems
pub fn init(kheap_start: usize, kheap_size: usize) {
    heap::init(kheap_start, kheap_size);
    physmem::init(kheap_start + kheap_size);
    vm::init(kheap_start + kheap_size + (4 << 20));
}
