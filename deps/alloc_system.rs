// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_name = "alloc_system"]
#![crate_type = "rlib"]

#![feature(allocator)]
#![feature(no_std)]
#![feature(staged_api)]

#![allocator]
#![no_std]
#![staged_api]

// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[allow(dead_code)]
const MIN_ALIGN: usize = 16;

extern {
    fn __rust_allocate(size: usize, align: usize) -> *mut u8;
    fn __rust_deallocate(ptr: *mut u8, old_size: usize, align: usize);
    fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> *mut u8;
    fn __rust_reallocate_inplace(ptr: *mut u8, old_size: usize, size: usize, align: usize) -> usize;
    fn __rust_usable_size(size: usize, align: usize) -> usize;
}
