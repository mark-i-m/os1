//! This module contains the outside interface with the kernels memory allocator.
//! rustc will look for these functions.
//!
//! The comments below are copied from the rust src code

#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(private_no_mangle_fns)] // If something goes wrong, try removing and see compiler says

use core::ptr;

use interrupts::no_preempt;
use memory::heap::{malloc, free, usable_size, print_stats};

/// Return a pointer to `size` bytes of memory aligned to `align`.
///
/// On failure, return a null pointer.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
#[no_mangle]
pub unsafe extern "C" fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    no_preempt(|| malloc(size, align))
}

#[no_mangle]
pub unsafe extern "C" fn __rust_allocate_zeroed(size: usize, align: usize) -> *mut u8 {
    let ret = no_preempt(|| malloc(size, align));

    if !ret.is_null() {
        ptr::write_bytes(ret, 0, size);
    }

    ret
}

/// Deallocates the memory referenced by `ptr`.
///
/// The `ptr` parameter must not be null.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[no_mangle]
pub unsafe extern "C" fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize) {
    no_preempt(|| free(ptr, old_size))
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// On failure, return a null pointer and leave the original allocation intact.
///
/// If the allocation was relocated, the memory at the passed-in pointer is
/// undefined after the call.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[no_mangle]
pub unsafe extern "C" fn __rust_reallocate(ptr: *mut u8,
                                           old_size: usize,
                                           size: usize,
                                           align: usize)
                                           -> *mut u8 {
    if size <= old_size {
        return ptr;
    }

    no_preempt(|| {
        let try_alloc = malloc(size, align);
        if try_alloc as usize != 0 {
            // copy old contents
            ptr::copy(ptr, try_alloc, old_size);

            // then free old ptr
            free(ptr, old_size);
        }

        try_alloc
    })
}

/// Resize the allocation referenced by `ptr` to `size` bytes.
///
/// If the operation succeeds, it returns `usable_size(size, align)` and if it
/// fails (or is a no-op) it returns `usable_size(old_size, align)`.
///
/// Behavior is undefined if the requested size is 0 or the alignment is not a
/// power of 2. The alignment must be no larger than the largest supported page
/// size on the platform.
///
/// The `old_size` and `align` parameters are the parameters that were used to
/// create the allocation referenced by `ptr`. The `old_size` parameter may be
/// any value in range_inclusive(requested_size, usable_size).
#[no_mangle]
pub unsafe extern "C" fn __rust_reallocate_inplace(ptr: *mut u8,
                                                   old_size: usize,
                                                   size: usize,
                                                   align: usize)
                                                   -> usize {

    no_preempt(|| usable_size(old_size, align))
}

/// Returns the usable size of an allocation created with the specified the
/// `size` and `align`.
#[no_mangle]
pub unsafe extern "C" fn __rust_usable_size(size: usize, align: usize) -> usize {
    no_preempt(|| usable_size(size, align))
}

/// Prints implementation-defined allocator statistics.
///
/// These statistics may be inconsistent if other threads use the allocator
/// during the call.
#[no_mangle]
pub unsafe extern "C" fn __rust_stats_print() {
    no_preempt(|| print_stats())
}
