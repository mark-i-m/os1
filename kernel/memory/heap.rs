//! This file contains the memory allocator used by the kernel. It is a thin wrapper around
//! smallheap.

use core::alloc::{GlobalAlloc, Layout, Opaque};
use core::cell::RefCell;

use smallheap::{self, Allocator};

use interrupts::no_interrupts;

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

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        no_interrupts(|| {
            self.heap
                .borrow_mut()
                .as_mut()
                .unwrap()
                .malloc(layout.size(), layout.align())
                .map(|p| p.as_ptr() as *mut Opaque)
                .unwrap_or(0 as *mut Opaque)
        })
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        no_interrupts(|| {
            self.heap
                .borrow_mut()
                .as_mut()
                .unwrap()
                .free(ptr as *mut u8, layout.size())
        })
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
