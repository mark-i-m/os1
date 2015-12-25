//! A simple vector implementation

use alloc::raw_vec::RawVec;

use core::ops::{Index, IndexMut};

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
        // TODO
    }

    /// Push the element to the end
    pub fn push(&mut self, val: T) {
        // TODO
    }

    /// Pop and return the last element
    pub fn pop(&mut self) -> T {
        // TODO
    }

    /// Return the number of elements
    pub fn size(&self) -> usize {
        self.len
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        // TODO
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        // TODO
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        // TODO
    }
}
