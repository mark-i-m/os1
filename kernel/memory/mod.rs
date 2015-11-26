// This module contains everything that has to do with memory

pub use self::vm::vmm_page_fault;

pub mod rust_alloc;

mod heap;
mod vm;

pub fn init(heap_start: usize, heap_end: usize,
            frames_start: usize, frames_end: usize) {
    heap::init(heap_start, heap_end);
    vm::init(frames_start, frames_end);
}
