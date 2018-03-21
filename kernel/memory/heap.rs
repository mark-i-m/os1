//! This file contains the memory allocator used by the rust_alloc module
//!
//! ***INTERRUPTS MUST BE OFF BEFORE RUNNING ANYTHING IN THIS FILE!!!!
//!
//! The implementation in this file is a simple first-fit allocator.
//!
//! Invariants:
//! * BLOCK_ALIGN = 4*size_of::<usize>()
//! * All blocks will be a multiple of BLOCK_ALIGN
//! * All blocks will be BLOCK_ALIGN-aligned
//!
//! Block structure:
//! * size
//! * forward pointer | free bits
//! * backward pointer | 0x0
//!   ...
//! * size (last word)
//!
//! When a block is free, the first and last words should match and be equal to the size of the
//! block. The forward pointer points to the head of the next free block. The backward pointer
//! points to the head of the previous free block. Since all blocks are at least 16B aligned, at
//! least the last 4 bits of all block addresses are 0. The last 4 bits of the forward pointer
//! should be all 1s if the block is free.
//!
//! When a block is in use, the whole block is usable for the user. Thus,
//! usable size and block size are equal.

#![allow(dead_code)]

use alloc::heap::{Alloc, AllocErr, Layout};

use core::mem::size_of;

use interrupts::{off, on};

/// A flag to turn on and off debugging output
const DEBUG: bool = false;

/// A const representing the minimum alignment of any block.
/// It is effectively a const, but it has to be initialized at startup because `size_of` is
/// non-const.
static mut BLOCK_ALIGN: usize = 0;

/// The start address of the kernel heap
static mut START: usize = 0;

/// The end address of the kernel heap. That is, the first address that is not in the heap.
static mut END: usize = 0;

/// A pointer to the first free heap block
static mut FREE_LIST: *mut Block = 0 as *mut Block;

// heap stats
/// The number of successful mallocs (for stats purposes)
static mut SUCC_MALLOCS: usize = 0;
/// The number of unsuccessful mallocs (for stats purposes)
static mut FAIL_MALLOCS: usize = 0;
/// The number of successful frees (for stats purposes)
static mut FREES: usize = 0;

/// A wrapper around the heap allocator
pub struct KernelAllocator;

unsafe impl<'a> Alloc for &'a KernelAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        off();
        let ret = malloc(layout.size(), layout.align());
        on();
        Ok(ret)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        off();
        let ret = free(ptr, layout.size());
        on();
        ret
    }
}

/// A struct representing a heap block.
///
/// It is empty because, by nature, heap blocks are variable size. Rather thsn give the compiler
/// a hard time, we can just a block fixed size and write methods that are aware of the situation.
/// Then, we just need to cast an address into a block pointer. This is a very C-ish solution,
/// but it makes life easy because a heap implementation usually has a lot of pointer arithmetic.
struct Block;

impl Block {
    /// Returns the first word of the block, which will contain the size if this is a free block
    unsafe fn get_head(&self) -> usize {
        *(self as *const Block as *const usize)
    }

    /// Sets the header to the given value
    unsafe fn set_head(&mut self, head: usize) {
        *(self as *mut Block as *mut usize) = head;
    }

    /// Returns the last word of the block, which will contain the size if this is a free block
    unsafe fn get_foot(&self) -> usize {
        let ptr: *const u8 = self as *const Block as *const u8;
        let size = self.get_size() - size_of::<usize>();

        // avoid overflows and weirdness
        if size >= END - START {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        *(ptr.offset(size as isize) as *const usize)
    }

    /// Sets the footer to the given value
    unsafe fn set_foot(&mut self, foot: usize) {
        let ptr: *mut u8 = self as *mut Block as *mut u8;
        let size = self.get_size() - size_of::<usize>();

        // avoid overflows and weirdness
        if size >= END - START {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        *(ptr.offset(size as isize) as *mut usize) = foot;
    }

    /// Gets the 4 free bits of the block
    unsafe fn get_free_bits(&self) -> u8 {
        (*(self as *const Block as *const usize).offset(1) & 0xF) as u8
    }

    /// Set the forward pointer (but not the free bits)
    unsafe fn set_next(&mut self, next: *mut Block) {
        *(self as *mut Block as *mut usize).offset(1) =
            ((next as usize) & !0xF) | (self.get_free_bits() as usize);
    }

    /// Set the backward pointer
    unsafe fn set_prev(&mut self, prev: *mut Block) {
        *(self as *mut Block as *mut usize).offset(2) = (prev as usize) & !0xF;
    }

    /// Returns true if the block is a valid free block.
    /// This method does a lot of sanity checiking
    pub unsafe fn is_free(&self) -> bool {
        // make sure the block addr is reasonable
        let self_usize = self as *const Block as usize;
        if self_usize < START || self_usize >= END || self_usize % BLOCK_ALIGN != 0 {
            return false;
        }

        let head = self.get_head();

        // make sure the block size is reasonable
        if head < BLOCK_ALIGN || head % BLOCK_ALIGN != 0 || head > END - START {
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

        if (forward_ptr < (START as *mut Block) && !forward_ptr.is_null())
            || forward_ptr >= (END as *mut Block)
            || (backward_ptr < (START as *mut Block) && !backward_ptr.is_null())
            || backward_ptr >= (END as *mut Block)
        {
            return false;
        }

        // check that the block is free
        self.get_free_bits() == 0xF
    }

    /// Set the free bits
    pub unsafe fn mark_free(&mut self) {
        *(self as *mut Block as *mut usize).offset(1) |= 0xF;
    }

    /// Clear the free bits.
    /// The block should already be removed from the free list
    pub unsafe fn mark_used(&mut self) {
        *(self as *mut Block as *mut usize).offset(1) &= !0xF; // bitwise !
    }

    /// Return the size of the block (user-usable data size)
    pub unsafe fn get_size(&self) -> usize {
        self.get_head()
    }

    /// Set the size of block
    pub unsafe fn set_size(&mut self, size: usize) {
        self.set_head(size);
        self.set_foot(size);
    }

    /// Returns the heap block immediately following this one
    pub unsafe fn get_contiguous_next(&self) -> *mut Block {
        let ptr: *const u8 = self as *const Block as *const u8;
        let size = self.get_size();

        // avoid overflows and weirdness
        if size >= END - START {
            panic!("Huge block at {:x}\n", ptr as usize);
        }

        ptr.offset(size as isize) as *mut Block
    }

    /// Returns the heap block immediately preceding this one
    /// or null if there is no valid previous block.
    pub unsafe fn get_contiguous_prev(&self) -> *mut Block {
        let prev_size_ptr = (self as *const Block as *const usize).offset(-1);

        // check that we are still in the heap
        if (prev_size_ptr as usize) < START {
            return 0 as *mut Block;
        }

        let prev_size = *prev_size_ptr;

        // check that the size is reasonable
        if prev_size >= END - START || prev_size == 0 {
            return 0 as *mut Block;
        }

        // attempt to get pointer to previous block
        let ptr: *const u8 = self as *const Block as *const u8;
        ptr.offset(-(prev_size as isize)) as *mut Block
    }

    /// Returns the forward ptr of the block (excluding the free bits)
    pub unsafe fn get_free_next(&self) -> *mut Block {
        (*(self as *const Block as *const usize).offset(1) & !0xF) as *mut Block
    }

    /// Returns the backward ptr of the block
    pub unsafe fn get_free_prev(&self) -> *mut Block {
        (*(self as *const Block as *const usize).offset(2) & !0xF) as *mut Block
    }

    /// Remove from free list. Clear forward/backward pointers
    pub unsafe fn remove(&mut self) {
        // make sure the block is free and valid
        if !self.is_free() {
            panic!(
                "Attempt to remove used block from free list: {:x}\n",
                self as *const Block as usize
            );
        }

        // swap pointers
        let next_ptr = self.get_free_next();
        let prev_ptr = self.get_free_prev();

        if !next_ptr.is_null() {
            (*next_ptr).set_prev(prev_ptr);
        }

        if !prev_ptr.is_null() {
            (*prev_ptr).set_next(next_ptr);
        } else {
            FREE_LIST = next_ptr;
        }

        self.set_next(0 as *mut Block);
        self.set_prev(0 as *mut Block);
    }

    /// Add to head of free list
    pub unsafe fn insert(&mut self) {
        let old_head = FREE_LIST;

        if !self.is_free() {
            print_stats();
            panic!(
                "Adding taken block to free list: {:x}",
                self as *const Block as usize
            );
        }

        FREE_LIST = self as *mut Block;

        self.set_next(old_head);
        self.set_prev(0 as *mut Block);

        if !old_head.is_null() {
            (*old_head).set_prev(self as *mut Block);
        }
    }

    /// Merge this block with the next block.
    /// Removes the second block from the free list before merging
    pub unsafe fn merge_with_next(&mut self) {
        // make sure both blocks are free and valid
        if !self.is_free() {
            panic!(
                "Attempt to merge non-free block 0x{:x} with next",
                self as *const Block as usize
            );
        }

        let next = self.get_contiguous_next();
        if !(*next).is_free() {
            panic!(
                "Attempt to merge 0x{:x} with non-free block 0x{:x}",
                self as *const Block as usize, next as usize
            );
        }

        // remove next block from free list
        (*next).remove();

        // make this block larger
        let new_size = self.get_size() + (*next).get_size();
        self.set_size(new_size);
    }

    /// Split the block so that it is the given size.
    /// Insert the new block into the free list.
    /// The block must be large enough to split (`>= 2*BLOCK_ALIGN`).
    /// The block must be free.
    /// The new size must be a multiple of `BLOCK_ALIGN`.
    pub unsafe fn split(&mut self, size: usize) {
        // make sure the block is free and valid
        if !self.is_free() {
            panic!(
                "Attempt to split non-free block: {:x}",
                self as *const Block as usize
            );
        }

        let old_size = self.get_size();

        // make sure the block is large enough
        if old_size < 32 {
            panic!(
                "Block is to small to split: {:x}",
                self as *const Block as usize
            );
        }

        if old_size < size + BLOCK_ALIGN {
            panic!(
                "Block is to small to split at {}: {:x}",
                size, self as *const Block as usize
            );
        }

        let new_block_size = old_size - size;

        // make this block smaller
        self.set_size(size);

        // set the next block size
        let new_block = self.get_contiguous_next();

        (*new_block).set_size(new_block_size);
        (*new_block).mark_free();
        (*new_block).set_next(0 as *mut Block);
        (*new_block).set_prev(0 as *mut Block);
        (*new_block).insert();
    }

    /// Returns true if this block matches the size and alignment.
    /// Returns the size at which the block should be split to obtain
    /// an aligned block; the second return value is meaningless if
    /// we return false.
    pub unsafe fn is_match(&self, size: usize, align: usize) -> (bool, usize) {
        let block_addr = self as *const Block as usize;
        let aligned = round_to_n(block_addr, align);

        // aligned part needs to be at least size bytes
        let split_size = aligned - block_addr;
        let matched = self.get_size() >= size + split_size;

        (matched, split_size)
    }
}

/// Round up to the nearest multiple of `BLOCK_ALIGN`
fn round_to_block_align(size: usize) -> usize {
    unsafe { round_to_n(size, BLOCK_ALIGN) }
}

/// Round the size to `n`. `n` must be a power of 2
fn round_to_n(size: usize, n: usize) -> usize {
    if size % n == 0 {
        size
    } else {
        size - (size % n) + n
    }
}

/// Initialize the kernel heap
pub fn init(start: usize, size: usize) {
    unsafe {
        // set BLOCK_ALIGN
        BLOCK_ALIGN = 4 * size_of::<usize>();

        // Round up to nearest multiple of 16
        START = round_to_block_align(start);

        // round END down
        END = (START + size) & !0xF;

        // bounds check
        if END <= START {
            panic!("No heap space");
        }

        // create first block and free list
        let first = START as *mut Block;

        (*first).set_size(END - START);
        (*first).mark_free();
        (*first).set_next(0 as *mut Block);
        (*first).set_prev(0 as *mut Block);

        FREE_LIST = first;

        bootlog!(
            "heap inited - start addr: 0x{:x}, end addr: 0x{:x}, {} bytes\n",
            START,
            END,
            END - START
        );
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
    if DEBUG {
        bootlog!("malloc {}, {} -> ", size, align);
    }

    if size == 0 {
        return 0 as *mut u8;
    }

    size = round_to_block_align(size);
    align = align.next_power_of_two();

    // get a free block
    let mut begin = FREE_LIST;
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
        if FREE_LIST.is_null() {
            panic!("Out of memory!");
        } else {
            // NOTE: fail for now because rustc does not know
            // how to handle failed mallocs
            panic!(
                "malloc({}, {}) -> *** malloc failed, rustc cannot handle this :( ***\n",
                size, align
            );
        }

        // update stats
        // FAIL_MALLOCS += 1;

        // return 0 as *mut u8;
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

    // mess up metadata so as to make life easier later
    (*begin).set_foot(0xFFFFFFFF);
    (*begin).set_head(0xFFFFFFFF);

    // update stats
    SUCC_MALLOCS += 1;

    if DEBUG {
        bootlog!("0x{:X}\n", begin as *const u8 as usize);
    }

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
    if DEBUG {
        bootlog!("free 0x{:X}, {}\n", ptr as usize, old_size);
    }

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

    // TODO: fix me
    // let prev_block = (*block).get_contiguous_prev();
    // if !prev_block.is_null() && (*prev_block).is_free() {
    //    (*prev_block).merge_with_next();
    // }

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
    unsafe {
        bootlog!("\nHeap stats\n----------\n");

        // Number of free blocks and amount of free memory
        let (num_free, size_free, size_used) = get_block_stats();
        bootlog!(
            "{} free blocks; {} bytes free, {} bytes used\n",
            num_free,
            size_free,
            size_used
        );

        // Number of mallocs and frees
        bootlog!(
            "Successfull mallocs: {}; Failed mallocs: {}; Frees: {}\n\n",
            SUCC_MALLOCS,
            FAIL_MALLOCS,
            FREES
        );
    }
}

/// Helper method to compute stats
unsafe fn get_block_stats() -> (usize, usize, usize) {
    // counters
    let mut num_free = 0;
    let mut size_free = 0;

    // loop through all blocks
    let mut begin = FREE_LIST;
    while !begin.is_null() {
        num_free += 1;
        size_free += (*begin).get_size();
        bootlog!("free {:X}: {} B\n", begin as usize, (*begin).get_size());
        begin = (*begin).get_free_next();
    }

    (num_free, size_free, END - START - size_free)
}
