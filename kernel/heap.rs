#![allow(dead_code)]
// This file contains the memory allocator used by the rust_alloc module
//
// The implementation in this file is a simple first-fit allocator.
//
// Invariants:
// * All blocks will be a multiple of 16B
// * All blocks will be 16B-aligned

extern crate core;

use core::mem::{size_of};
use core::option::Option::{self, Some, None};

static mut START: usize = 0;
static mut END: usize = 0;

static mut free_list: *mut Block = (0 as *mut Block);

// heap stats
static mut SUCC_MALLOCS: usize = 0;
static mut FAIL_MALLOCS: usize = 0;
static mut FREES:        usize = 0;

// memory block
// the last word of every block is its allocation size
#[repr(C, packed)]
struct Block {
    magic: usize, // If this is a free block, then it is magical: 0xCAFEFACE
    size: usize, // Includes the size of the footer
    next: *mut Block,
    prev: *mut Block,
}

impl Block {
    // METHODS FOR ALL BLOCKS

    // returns true if this is a valid free block
    fn is_free(&self) -> bool {
        self.magic == 0xCAFEFACE && self.size % 0x10 == 0
    }

    // get the prev block in memory
    //
    // note: this is distinct from the prev block in the free list
    // behavior is undefined if this is the last block.
    unsafe fn get_prev(&self) -> *mut Block {
        // check for corner cases
        if self.this() == START {
            panic!("Free block has no previous block: {}", self.this());
        }

        // get the addr of previous block's size
        let prev_foot: *const usize = (self.this() - core::mem::size_of::<usize>()) as *const usize;
        let prev_size = *prev_foot;

        // get previous block's addr
        (self.this() - prev_size) as *mut Block
    }

    // get the next block in memory
    //
    // note: this is distinct from the next block in the free list
    // behavior is undefined if this is the last block.
    unsafe fn get_next(&self) -> *mut Block {
        // check for corner cases
        if self.this() + self.size == END {
            panic!("Free block has no next block: {}", self.this());
        }

        (self.this() + self.size) as *mut Block
    }

    // Set the footer for this block. This method does not error checking, so
    // be careful!
    unsafe fn set_footer(&self, size: usize) {
        let footer = (self.this() + self.size - core::mem::size_of::<usize>()) as *mut usize;
        *footer = size;
    }

    #[inline]
    unsafe fn this(&self) -> usize {
        self as *const Block as usize
    }

    // METHODS FOR ONLY FREE BLOCKS

    // split the block into two blocks. The first block will be of the
    // given size. The block must be free
    unsafe fn split(&mut self, size: usize) {
        // check that the math works out
        if !self.is_free() {
            panic!("Attempt to split non-free block 0x{:X}",
                   (self as *const Block) as usize);
        }
        if round_to_16(size + core::mem::size_of::<usize>()) >= self.size {
            panic!("Splitting block that is too small: 0x{:X}, size {}",
                   (self as *const Block) as usize, size);
        }

        // get new block addr
        let new_size = round_to_16(size + core::mem::size_of::<usize>());

        // create new block and set magic bits
        let block = (self.this() + new_size) as *mut Block;
        (*block).size = self.size - new_size;
        (*block).set_footer((*block).size);

        // adjust this block's metadata
        self.size = new_size;

        // insert at tail of free list
        (*block).insert();
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
       self.size += (*next).size + core::mem::size_of::<usize>();

       // remove next block from free list
       (*next).remove();
    }

    // remove this block from the free list. The block
    // must be free. After this operation, the block's magic word is
    // set to 0xDEADBEEF.
    unsafe fn remove(&mut self) {
        if !self.is_free() {
            panic!("Attempt to remove non-free block from free list: {}", self.this());
        }

        // Set magic, so that this is definitely not a free block
        self.magic = 0xDEADBEEF;

        // get prev and next in free list
        // corner cases:
        // - if this is the head, prev = NULL
        // - if this is the tail, next = NULL

        if self.prev != (0 as *mut Block) {
            (*self.prev).next = self.next;
        } else {
            // remove the head of the list
            free_list = self.next;
        }

        if self.next != (0 as *mut Block) {
            // not the tail
            (*self.next).prev = self.prev;
        }
    }

    // METHODS FOR ONLY USED BLOCKS

    // inserts the given block at the head of the free list and sets
    // the magic bits. The block cannot already be free.
    unsafe fn insert(&mut self) {
        if self.is_free() {
            panic!("Attempt to insert a free block: {}", self.this());
        }

        // set magic bits
        self.magic = 0xCAFEFACE;

        // insert at head
        let next = free_list;

        free_list = self as *mut Block;
        self.next = next;

        if next != (0 as *mut Block) {
            (*next).prev = self.this() as *mut Block;
        }
    }

    // STATIC

    // Find a block that fits the bill
    unsafe fn find(size: usize, align: usize) -> Option<*mut Block> {
        let mut found = None;

        // loop through blocks until a good one is found
        let mut block_addr = free_list;

        while block_addr != (0 as *mut Block) {
            let block = &*block_addr;

            if Block::is_match(block, size, align) {
                found = Some(block_addr);
                break;
            }

            if (block_addr as usize) + block.size >= END {
                break;
            } else {
                block_addr = block.get_next();
            }
        }

        found
    }

    fn is_match(block: &Block, size: usize, align: usize) -> bool {
        // TODO: alignment

        block.size >= size
    }
}

// round up to the nearest multiple of 16
fn round_to_16(size: usize) -> usize {
    if (size & 0xF) == 0 { size } else { (size & !0xF) + 0x10 }
}

// Init the heap
pub fn init(start: usize, end: usize) {
    // Round to nearest multiple of 16
    unsafe {
        // round START up
        START = round_to_16(start);

        // round END down
        END = end & !0xF;

        // bounds check
        if END <= START {
            panic!("No heap space");
        }

        printf! ("In heap init\nstart addr: {:x}, end addr: {:x}, {} bytes\n",
                 START, END, END - START);
    }

    // create first block and free list
    unsafe {
        let first: &mut Block = &mut*(START as *mut Block);

        first.magic = 0xCAFEFACE;
        first.size = END - START;
        first.next = 0 as *mut Block;
        first.prev = 0 as *mut Block;
        first.set_footer(first.size);

        free_list = START as *mut Block;
    }

    print_stats();
}

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
pub unsafe fn malloc(size: usize, align: usize) -> *mut u8 {
    // TODO: alignment

    // TODO: lock here

    // get free block
    let block_addr = Block::find(size, align);

    match block_addr {
        None => {
            // check for out-of-memory; that is, empty free list
            if free_list == (0 as *mut Block) {
                panic!("Out of memory!");
            }

            // update stats
            FAIL_MALLOCS += 1;

            // TODO: unlock here

            // return failure
            0 as *mut u8
        }
        Some(addr) => {
            let block: &mut Block = &mut*addr;

            // Split the block if it is too big
            if block.size > round_to_16(size + core::mem::size_of::<usize>()) {
                block.split(size);
            }

            // Remove the block from the free list
            block.remove();

            // update stats
            SUCC_MALLOCS += 1;

            // TODO: unlock here

            // return ptr
            addr as *mut u8
        }
    }
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
pub unsafe fn free(ptr: *mut u8, old_size: usize) {
    FREES += 1;
    //TODO
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
pub fn usable_size(size: usize, align: usize) -> usize {
    let usize_size = core::mem::size_of::<usize>();
    round_to_16(size + usize_size) - usize_size
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
pub fn print_stats() {
    printf!("\nHeap stats\n----------\n");

    // Number of free blocks and amount of free memory
    let (num_free, size_free, size_used) = get_block_stats();
    printf!("{} free blocks; {} bytes free, {} bytes used\n",
            num_free, size_free, size_used);

    // Number of mallocs and frees
    unsafe {
        printf!("Successfull mallocs: {}; Failed mallocs: {}; Frees: {}\n\n",
                SUCC_MALLOCS, FAIL_MALLOCS, FREES);
    }
}

// Helper methods to compute stats
fn get_block_stats() -> (usize, usize, usize) {
    // counters
    let mut num_free = 0;
    let mut size_free = 0;

    // TODO: lock here

    // loop through all blocks
    unsafe{
        let mut block_addr = free_list as *mut Block;

        while block_addr != (0 as *mut Block) {
            let block = &*block_addr;

            num_free += 1;
            size_free += block.size;

            if block.next == (0 as *mut Block) {
                break;
            } else {
                block_addr = block.next;
            }
        }
    }

    // TODO: unlock here

    (num_free, size_free, unsafe { END - START - size_free })
}
