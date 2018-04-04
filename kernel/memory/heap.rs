//! This file contains the memory allocator used by the kernel. It is a thin wrapper around
//! smallheap.

use alloc::heap::{Alloc, AllocErr, Layout};
use core::cell::RefCell;

use smallheap::Allocator;

use interrupts::{off, on};

/// A wrapper around the heap allocator for use as the `global_allocator`.
pub struct KernelAllocator {
    heap: RefCell<Option<Allocator>>,
}

impl KernelAllocator {
    pub const fn new() -> Self {
        KernelAllocator {
            heap: RefCell::new(None),
        }
    }

    pub fn set_heap(&mut self, heap: Allocator) {
        *self.heap.borrow_mut() = Some(heap);
    }
}

unsafe impl<'a> Alloc for &'a KernelAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        off();
        let ret = self.heap
            .borrow_mut()
            .as_mut()
            .unwrap()
            .malloc(layout.size(), layout.align());
        on();
        Ok(ret)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        off();
        let ret = self.heap
            .borrow_mut()
            .as_mut()
            .unwrap()
            .free(ptr, layout.size());
        on();
        ret
    }
}

/// Initialize the kernel heap
pub fn init(allocator: &mut KernelAllocator, start: usize, size: usize) {
    let heap = unsafe { Allocator::new(start as *mut u8, size) };
    let free_size = heap.size();

    allocator.set_heap(heap);

    bootlog!(
        "heap inited - start addr: 0x{:x}, end addr: 0x{:x}, {} bytes\n",
        start,
        start + size,
        free_size
    );
}
