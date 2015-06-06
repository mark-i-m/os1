// This module contains everything that has to do with memory

pub mod rust_alloc;
mod heap;

pub fn init(start: usize, end: usize) {
    heap::init(start, end);
    // TODO: init virt. mem here also...
}
