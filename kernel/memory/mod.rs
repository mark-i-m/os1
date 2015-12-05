// This module contains everything that has to do with memory

pub use self::virtmem::{vmm_page_fault, AddressSpace};

pub mod rust_alloc;

mod heap;
mod physmem;
mod virtmem;
mod regionmap;

pub fn init(heap_start: usize, heap_size: usize) {
    heap::init(heap_start, heap_size);
    physmem::init(unsafe { self::heap::END });
}
