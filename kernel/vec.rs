//! A simple vector implementation

#![allow(dead_code)]

use alloc::raw_vec::RawVec;

use core::ops::{Index, IndexMut};

use core::ptr;

/// A vector implementation.
/// - Amortized O(1) push to end
/// - O(1) pop from end
/// - O(1) access to any element
/// - Automatic memory management
/// - Indexable with [] operator
pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vec<T> {
    /// Create a new empty vector
    pub fn new() -> Vec<T> {
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    /// Push the element to the end
    pub fn push(&mut self, val: T) {
        // resize if needed
        if self.len == self.buf.cap() {
            self.buf.double();
        }

        // put at the end
        unsafe {
            ptr::write(self.buf.ptr().offset(self.len as isize), val);
        }
        self.len += 1;
    }

    /// Pop and return the last element
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(ptr::read(self.buf.ptr().offset(self.len as isize)))
            }
        }
    }

    /// Return the number of elements
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        if index >= self.len {
            panic!("Out of bounds! {} out of {}", index, self.len);
        }

        unsafe {
            &*self.buf.ptr().offset(index as isize)
        }
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        if index >= self.len {
            panic!("Out of bounds! {} out of {}", index, self.len);
        }

        unsafe {
            &mut *self.buf.ptr().offset(index as isize)
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        // destruct all
        for _ in 0..self.len {
            let _ = self.pop();
        }

        // deallocate
        self.buf.shrink_to_fit(0);
    }
}
