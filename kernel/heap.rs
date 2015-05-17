#![allow(dead_code)]
// This file contains the memory allocator used by the rust_alloc module
//
// The implementation in this file is a simple first-fit allocator.
//
// Invariants:
// * All blocks will be a multiple of 16B
// * All blocks will be 16B-aligned
//
// TODO: lock free list

extern crate core;
use core::mem::{transmute_copy};

static mut START: usize = 0;
static mut END: usize = 0;

static mut free_list: *mut Block = (0 as *mut Block);

// memory block
#[repr(C, packed)]
struct Block {
    magic: usize, // If this is a free block, then it is magical: 0xCAFEFACE
    size: usize,
    next: *mut Block,
    prev: *mut Block,
}

impl Block {
    // returns true if this is a valid free block
    fn is_free(&self) -> bool { self.magic == 0xCAFEFACE && self.size % 0x10 == 0 }

    // split the block into two blocks. The first block will be of the
    // given size. size must be a multiple of 16. The block must be free
    unsafe fn split(&mut self, size: usize) {
        // check that the math works out
        if !self.is_free() {
            panic!("Attempt to split non-free block 0x{:X}",
                   (self as *const Block) as usize);
        }
        if !(size % 0x10 == 0) {
            panic!("Size is not a multiple of 16: {}", size);
        }
        if size >= self.size {
            panic!("Splitting block that is too small: 0x{:X}, size {}",
                   (self as *const Block) as usize, size);
        }

        // get new block addr
        let this: usize = self as *const Block as usize;
        let new_addr: *const Block = (this + size) as *const Block;

        // create new block and set magic bits
        let mut block: Block = core::mem::transmute_copy(&*new_addr);
        block.magic = 0xCAFEFACE;
        block.size = self.size - size;

        // adjust this block's metadata
        self.size = size;

        // insert at tail of free list
        Block::free_insert(&mut block);
    }

    // coalesce this block with the next one. The two blocks must be free
    unsafe fn combine(&mut self) {
        // check that both are free!
        if !self.is_free() {
            panic!("Attempt to coalesce non-free block 0x{:X}",
                   (self as *const Block) as usize);
        } else if !(*self.get_next()).is_free() {
            panic!("Attempt to coalesce non-free block 0x{:X}",
                   self.get_next() as *const Block as usize);
        }

        let next = self.get_next();

       // increase the size of this block
       self.size += (*next).size;

       // remove next block from free list
       (*next).remove();
    }

    // remove this block from the free list and return it. The block
    // must be free. After this operation, the block's magic word is
    // set to 0xDEADBEEF. The block must be free.
    unsafe fn remove(&mut self) {
        // TODO
    }

    // inserts the given block at the tail of the free list and sets
    // the magic bits. The block cannot already be free.
    unsafe fn insert(&mut self, block: &mut Block) {
        //TODO
    }

    // get the prev block in memory
    // NOTE: this is distinct from the prev block in the free list
    // behavior is undefined if this is the last block.
    unsafe fn get_prev(&self) -> *mut Block {
        //TODO
    }

    // get the next block in memory
    // NOTE: this is distinct from the next block in the free list
    // behavior is undefined if this is the last block.
    unsafe fn get_next(&self) -> *mut Block {
        if !self.is_free() {
            panic!("Attempt to get next block of non-free block 0x{:X}",
                   (self as *const Block) as usize);
        }

        //TODO: fixme
        // get address of next block
        let this = self as *const Block as usize;
        let next_block: Block =
            core::mem::transmute_copy(&*((this + self.size) as *mut Block));

        &mut next_block
    }
}

// Init the heap
pub fn init(start: usize, end: usize) {
    // Round to nearest multiple of 16
    unsafe {
        // round START up
        START = if (start & 0xF) == 0 {start} else {(start & !0xF) + 0x10};

        // round END down
        END = end & !0xF;

        // bounds check
        if END <= START {
            panic!("No heap space");
        }

        printf! ("In heap init\nstart addr: {:x}, end addr: {:x}\n", START, END);
    }

    // TODO: create first block and free list
}

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
pub unsafe fn malloc(size: usize, align: usize) -> *mut u8 {
    0 as *mut u8 // TODO
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
pub unsafe fn free(ptr: *mut u8, old_size: usize) {

}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
pub fn usable_size(size: usize, align: usize) -> usize {
    0 // TODO
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
pub fn print_stats() {

}
