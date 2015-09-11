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

use super::super::interrupts::{on, off};

const DEBUG: bool = false;

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
            panic!("Free block has no previous block: {:x}", self.this());
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
            panic!("Free block has no next block: {:x}", self.this());
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
    // given usable size. The block must be free
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

        // get new size of original block
        let new_size = round_to_16(size + core::mem::size_of::<usize>());

        // create new block
        let block = (self.this() + new_size) as *mut Block;
        (*block).size = self.size - new_size;
        (*block).set_footer((*block).size);

        // adjust this block's metadata
        self.size = new_size;
        self.set_footer(new_size);

        // insert at tail of free list
        (*block).insert();
    }

    // coalesce this block with the next one. The two blocks must be free
    unsafe fn combine(&mut self) {
        // check that both are free!
        if !self.is_free() {
            panic!("Attempt to coalesce non-free block 0x{:X} with next",
                   (self as *const Block) as usize);
        } else if !(*self.get_next()).is_free() {
            print_stats();
            panic!("Attempt to coalesce non-free block 0x{:X} with prev",
                   self.get_next() as *const Block as usize);
        }

        let next = self.get_next();

       // increase the size of this block
       self.size += (*next).size;

       // remove next block from free list
       (*next).remove();

       // adjust metadata
       self.set_footer(self.size);
    }

    // remove this block from the free list. The block
    // must be free. After this operation, the block's magic word is
    // set to 0xDEADBEEF.
    unsafe fn remove(&mut self) {
        if !self.is_free() {
            panic!("Attempt to remove non-free block from free list: {:x}", self.this());
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
            panic!("Attempt to insert a free block: {:X}", self.this());
        }

        // set magic bits
        self.magic = 0xCAFEFACE;

        // insert at head
        let next = free_list;

        free_list = self as *mut Block;
        self.next = next;

        self.prev = 0 as *mut Block;

        if next != (0 as *mut Block) {
            (*next).prev = self.this() as *mut Block;
        }
    }

    // STATIC

    // Find a block that fits the bill
    unsafe fn find(size: usize, align: usize) -> Option<*mut Block> {
        // loop through blocks until a good one is found
        let mut block_addr = free_list;

        while block_addr != (0 as *mut Block) {
            let block = &*block_addr;

            // sanity check
            if block_addr as usize >= END {
                panic!("Free block beyond END!");
            }

            // return the first matching block
            if Block::is_match(block, size, align) {
                return Some(block_addr);
            }

            block_addr = block.next;
        }

        None
    }

    fn is_match(block: &Block, size: usize, align: usize) -> bool {
        let block_usize = block as *const Block as usize;
        let aligned = round_to_n(block_usize, align);

        // total size - leftovers >= new_block and its metadata
        block.size - (aligned - block_usize) >= size
    }
}

// round up to the nearest multiple of 16
fn round_to_16(size: usize) -> usize {
    round_to_n(size, 16)
}

// n must be a power of 2
fn round_to_n(size: usize, n: usize) -> usize {
    if size % n == 0 { size } else { size - (size % n) + n }
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

        // create first block and free list
        let first: &mut Block = &mut*(START as *mut Block);

        first.magic = 0xCAFEFACE;
        first.size = END - START;
        first.next = 0 as *mut Block;
        first.prev = 0 as *mut Block;
        first.set_footer(first.size);

        free_list = START as *mut Block;

        bootlog! ("Heap inited: start 0x{:x}, end 0x{:x}, {} bytes\n",
                  START, END, END - START);
    }

    // print_stats();
}

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
pub unsafe fn malloc(size: usize, align: usize) -> *mut u8 {
    // disable interrutps
    off();

    if DEBUG {printf!("malloc {}, {} -> ", size, align);}

    // get free block
    let align_rounded = align.next_power_of_two();
    let size_rounded = round_to_16(size + core::mem::size_of::<usize>());
    let block_addr = Block::find(size_rounded, align_rounded);

    match block_addr {
        None => {
            // check for out-of-memory; that is, empty free list
            if free_list == (0 as *mut Block) {
                panic!("Out of memory!");
            }

            // update stats
            FAIL_MALLOCS += 1;

            // enable interrupts
            on();

            // return failure
            0 as *mut u8
        }
        Some(addr) => {
            let mut block: &mut Block = &mut*addr;

            // check if the aligned block is in the middle
            let addr_rounded = round_to_n(addr as usize, align_rounded);
            if addr_rounded > addr as usize {
                block.split(addr_rounded - (addr as usize) - core::mem::size_of::<usize>());
                block = &mut*block.get_next();
            }

            // Split the block if it is too big
            if block.size > size_rounded {
                block.split(size);
            }

            // Remove the block from the free list
            block.remove();

            // update stats
            SUCC_MALLOCS += 1;

            // enable interrutps
            on();

            if DEBUG {printf!("0x{:X}\n", block.this());}

            // return ptr
            block.this() as *mut u8
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

    if DEBUG {printf!("free 0x{:X}, {}\n", ptr as usize, old_size);}

    // check input
    if ptr == (0 as *mut u8) {
        panic!("Attempt to free NULL ptr");
    }

    let block: &mut Block = &mut*(ptr as *mut Block);

    if block.is_free() {
        panic!("Double free: {}", ptr as usize);
    }

    let usize_size = core::mem::size_of::<usize>();
    let true_size = round_to_16(old_size + usize_size);

    block.size = true_size;
    block.set_footer(block.size); // just in case

    // disable interrupts
    off();

    // update stats
    FREES += 1;

    // insert into free list
    block.insert();

    // attempt coalescing
    let prev_free = if ptr as usize != START {
        (*block.get_prev()).is_free()
    } else {
        false
    };

    let next_free = if ptr as usize + block.size < END {
        (*block.get_next()).is_free()
    } else {
        false
    };

    if next_free {
        block.combine();
    }

    if prev_free {
        (*block.get_prev()).combine();
    }

    // enable interrupts
    on();

    //print_stats();
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
#[allow(unused_variables)] // align has no impact on size in this implementation
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

    // disable interrupts
    off();

    // loop through all blocks
    unsafe{
        let mut block_addr = free_list as *mut Block;

        while block_addr != (0 as *mut Block) {
            let block = &*block_addr;

            num_free += 1;
            size_free += block.size;
            printf!("free {:X}: {} B\n", block_addr as usize, block.size);

            if block.next == (0 as *mut Block) {
                break;
            } else {
                block_addr = block.next;
            }
        }
    }

    // enable interrupts
    on();

    (num_free, size_free, unsafe { END - START - size_free })
}
