// This module contains everything that has to do with memory

pub use self::vm::vmm_page_fault;

pub use self::vm::AddressSpace;

pub use self::tss::esp0;

pub mod rust_alloc;

mod heap;
mod vm;
mod tss;

pub fn init(heap_start: usize, heap_end: usize,
            frames_start: usize, frames_end: usize) {
    tss::init();
    heap::init(heap_start, heap_end);
    vm::init(frames_start, frames_end);
}
