#![allow(dead_code)]
// This file contains the memory allocator used by the rust_alloc module
//
// ***INTERRUPTS MUST BE OFF BEFORE RUNNING ANYTHING IN THIS FILE!!!!
//
// The implementation in this file is a simple first-fit allocator.
//
// Invariants:
// * BLOCK_ALIGN = 4*size_of::<usize>()
// * All blocks will be a multiple of BLOCK_ALIGN
// * All blocks will be BLOCK_ALIGN-aligned
//
// Block structure:
// * size
// * forward pointer | free bits
// * backward pointer | 0x0
//   ...
// * size (last word)
//
// When a block is free, the first and last words should match and be equal to the size of the
// block. The forward pointer points to the head of the next free block. The backward pointer
// points to the head of the previous free block. Since all blocks are at least 16B aligned, at
// least the last 4 bits of all block addresses are 0. The last 4 bits of the forward pointer
// should be all 1s if the block is free.
//
// When a block is in use, the whole block is usable for the user. Thus,
// usable size and block size are equal.

use core::mem::{size_of};
use core::isize;

const DEBUG: bool = false;

static mut BLOCK_ALIGN: usize = 0;

static mut START: usize = 0;
static mut END: usize = 0;

static mut free_list: *mut Block = 0 as *mut Block;

// heap stats
static mut SUCC_MALLOCS: usize = 0;
static mut FAIL_MALLOCS: usize = 0;
static mut FREES:        usize = 0;

// memory block
struct Block;

impl Block {
    // returns the first word of the block
    unsafe fn get_head(&self) -> usize {
        *(self as *const Block as *const usize)
    }

    // sets the header to the given value
    unsafe fn set_head(&mut self, head: usize) {
        *(self as *mut Block as *mut usize) = head;
    }

    // returns the last word of the block
    unsafe fn get_foot(&self) -> usize {
        let ptr: *const u8 = self as *const Block as *const u8;
        let size = self.get_size() - size_of::<usize>();

        // avoid overflows and weirdness
        if size >= (isize::MAX as usize) {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        *(ptr.offset(size as isize) as *const usize)
    }

    // sets the footer to the given value
    unsafe fn set_foot(&mut self, foot: usize) {
        let ptr: *mut u8 = self as *mut Block as *mut u8;
        let size = self.get_size() - size_of::<usize>();

        // avoid overflows and weirdness
        if size >= (isize::MAX as usize) {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        *(ptr.offset(size as isize) as *mut usize) = foot;
    }

    // gets the 4 free bits of the block
    unsafe fn get_free_bits(&self) -> u8 {
        (*(self as *const Block as *const usize).offset(1) & 0xF) as u8
    }

    // set the forward pointer (but not the free bits)
    unsafe fn set_next(&mut self, next: *mut Block) {
        *(self as *mut Block as *mut usize).offset(1) =
            ((next as usize) & !0xF) | (self.get_free_bits() as usize);
    }

    // set the backward pointer
    unsafe fn set_prev(&mut self, prev: *mut Block) {
        *(self as *mut Block as *mut usize).offset(2) = (prev as usize) & !0xF;
    }

    // returns true if the block is a valid free block
    pub unsafe fn is_free(&self) -> bool {
        let head = self.get_head();

        if head < BLOCK_ALIGN {
            return false;
        }

        let foot = self.get_foot();

        // check sentinels
        if head != foot {
            return false;
        }

        // check that free list pointers are valid
        let forward_ptr = self.get_free_next();
        let backward_ptr = self.get_free_prev();

        if (forward_ptr < (START as *mut Block) &&
            !forward_ptr.is_null()) ||
           forward_ptr >= (END as *mut Block) ||
           (backward_ptr < (START as *mut Block) &&
            !backward_ptr.is_null()) ||
           backward_ptr >= (END as *mut Block){
            return false;
        }

        // check that the block is BLOCK_ALIGN aligned
        if (self as *const Block as usize) % BLOCK_ALIGN != 0 {
            return false;
        }

        // check that the block is free
        self.get_free_bits() == 0xF
    }

    // set the free bits
    pub unsafe fn mark_free(&mut self) {
        *(self as *mut Block as *mut usize).offset(1) |= 0xF;
    }

    // clear the free bits
    // the block should already be removed from the free list
    pub unsafe fn mark_used(&mut self) {
        *(self as *mut Block as *mut usize).offset(1) &= !0xF; // bitwise !
    }

    // return the size of the block (user-usable data size)
    pub unsafe fn get_size(&self) -> usize {
        self.get_head()
    }

    // set the size of block
    pub unsafe fn set_size(&mut self, size: usize) {
        self.set_head(size);
        self.set_foot(size);
    }

    // returns the heap block immediately following this one
    pub unsafe fn get_contiguous_next(&self) -> *mut Block {
        let ptr: *const u8 = self as *const Block as *const u8;
        let size = self.get_size();

        // avoid overflows and weirdness
        if size >= (isize::MAX as usize) {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        ptr.offset(size as isize) as *mut Block
    }

    // returns the heap block immediately preceding this one
    pub unsafe fn get_contiguous_prev(&self) -> *mut Block {
        (self as *const Block as *const usize).offset(-1) as *mut Block
    }

    // returns the forward ptr of the block
    // (excluding the free bits)
    pub unsafe fn get_free_next(&self) -> *mut Block {
        (*(self as *const Block as *const usize).offset(1) & !0xF) as *mut Block
    }

    // returns the backward ptr of the block
    pub unsafe fn get_free_prev(&self) -> *mut Block {
        (*(self as *const Block as *const usize).offset(2) & !0xF) as *mut Block
    }

    // remove from free list
    // clear forward/backward pointers
    pub unsafe fn remove(&mut self) {
        // make sure the block is free and valid
        if !self.is_free() {
            panic!("Attempt to remove used block from free list: {:x}\n",
                   self as *const Block as usize);
        }

        // swap pointers
        let next_ptr = self.get_free_next();
        let prev_ptr = self.get_free_prev();

        if !next_ptr.is_null() {
            (*next_ptr).set_prev(prev_ptr);
        }

        if !prev_ptr.is_null() {
            (*prev_ptr).set_next(next_ptr);
        }

        self.set_next(0 as *mut Block);
        self.set_prev(0 as *mut Block);
    }

    // add to head of free list
    pub unsafe fn insert(&mut self) {
        let old_head = free_list;

        if !self.is_free() {
            panic!("Adding taken block to free list: {:x}",
                  self as *const Block as usize);
        }

        free_list = self as *mut Block;

        self.set_next(old_head);
        self.set_prev(0 as *mut Block);

        if !old_head.is_null() {
            (*old_head).set_prev(self as *mut Block);
        }
    }

    // merge with next
    // removes the second block from the free list before merging
    pub unsafe fn merge_with_next(&mut self) {
        // make sure both blocks are free and valid
        if !self.is_free() {
            panic!("Attempt to merge non-free block: {:x}",
                  self as *const Block as usize);
        }

        let next = self.get_contiguous_next();
        if !(*next).is_free() {
            panic!("Attempt to merge with non-free block: {:x}",
                  self as *const Block as usize);
        }

        // remove next block from free list
        (*next).remove();

        // make this block larger
        let new_size = self.get_size() + (*next).get_size();
        self.set_size(new_size);
    }

    // split the block so that it is the given size
    // inserts the new block into the free list
    // the block must be large enough to split (>= 2*BLOCK_ALIGN)
    // the block must be free
    // the new size must be a multiple of BLOCK_ALIGN
    pub unsafe fn split(&mut self, size: usize) {
        // make sure the block is free and valid
        if !self.is_free() {
            panic!("Attempt to split non-free block: {:x}",
                  self as *const Block as usize);
        }

        let old_size = self.get_size();

        // make sure the block is large enough
        if old_size < 32 {
            panic!("Block is to small to split: {:x}",
                  self as *const Block as usize);
        }

        if old_size < size + BLOCK_ALIGN {
            panic!("Block is to small to split at {}: {:x}",
                  size, self as *const Block as usize);
        }

        let new_block_size = old_size - size;

        // make this block smaller
        self.set_size(size);

        // set the next block size
        let new_block = self.get_contiguous_next();

        (*new_block).set_size(new_block_size);
        (*new_block).mark_free();
        (*new_block).insert();
    }

    // returns true if this block matches the size and alignment
    // returns the size it which the block should be split to obtain
    //   an aligned block; the second return value is meaningless if
    //   we return false.
    pub unsafe fn is_match(&self, size: usize, align: usize) -> (bool, usize) {
        let block_addr = self as *const Block as usize;
        let aligned = round_to_n(block_addr, align);

        // aligned part needs to be at least size bytes
        let split_size = aligned - block_addr;
        let matched = self.get_size() >= size + split_size;

        (matched, split_size)
    }
}

// round up to the nearest multiple of BLOCK_ALIGN
fn round_to_block_align(size: usize) -> usize {
    unsafe {
        round_to_n(size, BLOCK_ALIGN)
    }
}

// n must be a power of 2
fn round_to_n(size: usize, n: usize) -> usize {
    if size % n == 0 { size } else { size - (size % n) + n }
}

// Init the heap
pub fn init(start: usize, end: usize) {
    unsafe {
        // set BLOCK_ALIGN
        BLOCK_ALIGN = 4 * size_of::<usize>();

        // Round up to nearest multiple of 16
        START = round_to_block_align(start);

        // round END down
        END = end & !0xF;

        // bounds check
        if END <= START {
            panic!("No heap space");
        }

        bootlog! ("heap start addr: {:x}, end addr: {:x}, {} bytes\n",
                 START, END, END - START);

        // create first block and free list
        let first = START as *mut Block;

        (*first).set_size(END - START);
        (*first).mark_free();
        (*first).set_next(0 as *mut Block);
        (*first).set_prev(0 as *mut Block);

        free_list = first;
    }
}

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
pub unsafe fn malloc(mut size: usize, mut align: usize) -> *mut u8 {
    if DEBUG {bootlog!("malloc {}, {} -> ", size, align);}

    if size == 0 {
        return 0 as *mut u8;
    }

    size = round_to_block_align(size);
    align = align.next_power_of_two();

    // get a free block
    let mut begin = free_list;
    let mut matched = false;
    let mut split_size = 0;
    while !begin.is_null() {
        let (matched_, split_size_) = (*begin).is_match(size, align);
        matched = matched_;
        split_size = split_size_;
        if matched {
            break;
        }
        begin = (*begin).get_free_next();
    }

    // no blocks found
    if !matched {
        if free_list.is_null() {
            panic!("Out of memory!");
        }

        // update stats
        FAIL_MALLOCS += 1;

        return 0 as *mut u8;
    }

    // split block if needed
    if split_size > 0 {
        (*begin).split(split_size);
        begin = (*begin).get_contiguous_next();
    }

    // split block to fit
    if (*begin).get_size() > size {
        (*begin).split(size);
    }

    // make block used
    (*begin).remove();
    (*begin).mark_used();

    // update stats
    SUCC_MALLOCS += 1;

    if DEBUG {bootlog!("0x{:X}\n", begin as *const u8 as usize);}

    begin as *mut u8
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
pub unsafe fn free(ptr: *mut u8, mut old_size: usize) {

    if DEBUG {bootlog!("free 0x{:X}, {}\n", ptr as usize, old_size);}

    // check input
    if ptr == (0 as *mut u8) {
        panic!("Attempt to free NULL ptr");
    }

    let block: *mut Block = ptr as *mut Block;

    if (*block).is_free() {
        panic!("Double free: {}", ptr as usize);
    }

    // get actual size of block
    old_size = round_to_block_align(old_size);

    // Create block and insert into free list
    (*block).set_size(old_size);
    (*block).mark_free();
    (*block).set_next(0 as *mut Block);
    (*block).set_prev(0 as *mut Block);
    (*block).insert();

    // attempt coalescing
    let next_block = (*block).get_contiguous_next();
    if (*next_block).is_free() {
        (*block).merge_with_next();
    }

    let prev_block = (*block).get_contiguous_prev();
    if (*prev_block).is_free() {
        (*prev_block).merge_with_next();
    }

    // update stats
    FREES += 1;
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
#[allow(unused_variables)] // align has no impact on size in this implementation
pub fn usable_size(size: usize, align: usize) -> usize {
    round_to_block_align(size)
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
pub fn print_stats() {
    unsafe{
        bootlog!("\nHeap stats\n----------\n");

        // Number of free blocks and amount of free memory
        let (num_free, size_free, size_used) = get_block_stats();
        bootlog!("{} free blocks; {} bytes free, {} bytes used\n",
                num_free, size_free, size_used);

        // Number of mallocs and frees
        bootlog!("Successfull mallocs: {}; Failed mallocs: {}; Frees: {}\n\n",
                SUCC_MALLOCS, FAIL_MALLOCS, FREES);
    }
}

// Helper methods to compute stats
unsafe fn get_block_stats() -> (usize, usize, usize) {
    // counters
    let mut num_free = 0;
    let mut size_free = 0;

    // loop through all blocks
    let mut begin = free_list;
    while !begin.is_null() {
        num_free += 1;
        size_free += (*begin).get_size();
        bootlog!("free {:X}: {} B\n", begin as usize, (*begin).get_size());
        begin = (*begin).get_free_next();
    }

    (num_free, size_free, END - START - size_free)
}

