#![allow(dead_code)]
// This file contains the memory allocator used by the rust_alloc module
//
// TODO: maybe a buddy allocator?
//
// The comments below come from the rust src code

// Init the heap
pub fn init() {
    printf! ("In heap init\n");
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
pub unsafe fn free(ptr: *mut u8) {

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
