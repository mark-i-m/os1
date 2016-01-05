//! A simple vector implementation

#![allow(dead_code)]

use alloc::heap::EMPTY;
use alloc::raw_vec::RawVec;

use core::intrinsics::{arith_offset, assume};
use core::mem;
use core::ops::{Index, IndexMut, Deref, DerefMut};
use core::ptr;
use core::slice;

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

/// An iterator that moves out of a vector modified from the stdlib
pub struct IntoIter<T> {
    _buf: RawVec<T>,
    ptr: *const T,
    end: *const T,
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

    /// Shorten a vector to be `len` elements long, dropping excess elements.
    ///
    /// If `len` is greater than the vector's current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = vec![1, 2, 3, 4, 5];
    /// vec.truncate(2);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            // drop any extra elements
            while len < self.len {
                // decrement len before the read(), so a panic on Drop doesn't
                // re-drop the just-failed value.
                self.len -= 1;
                ptr::read(self.buf.ptr().offset(self.len as isize));
            }
        }
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

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let p = self.buf.ptr();
            assume(!p.is_null());
            slice::from_raw_parts(p, self.len)
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.buf.ptr();
            assume(!ptr.is_null());
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Vec<T> {
        let mut v = Vec::new();
        for x in &**self {
            v.push(x.clone());
        }
        v
    }

    fn clone_from(&mut self, other: &Vec<T>) {
        // drop all elements
        self.truncate(0);

        // overwrite
        for x in &**other {
            self.push(x.clone());
        }
    }
}

/// Barrowed from libstd
impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Creates a consuming iterator, that is, one that moves each value out of
    /// the vector (from start to end). The vector cannot be used after calling
    /// this.
    ///
    /// # Examples
    ///
    /// ```
    /// let v = vec!["a".to_string(), "b".to_string()];
    /// for s in v.into_iter() {
    ///     // s has type String, not &String
    ///     println!("{}", s);
    /// }
    /// ```
    #[inline]
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let ptr = self.buf.ptr();
            assume(!ptr.is_null());
            let begin = ptr as *const T;
            let end = if mem::size_of::<T>() == 0 {
                arith_offset(ptr as *const i8, self.len() as isize) as *const T
            } else {
                ptr.offset(self.len() as isize) as *const T
            };
            let buf = ptr::read(&self.buf);
            mem::forget(self);
            IntoIter {
                _buf: buf,
                ptr: begin,
                end: end,
            }
        }
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            if self.ptr == self.end {
                None
            } else {
                if mem::size_of::<T>() == 0 {
                    // purposefully don't use 'ptr.offset' because for
                    // vectors with 0-size elements this would return the
                    // same pointer.
                    self.ptr = arith_offset(self.ptr as *const i8, 1) as *const T;

                    // Use a non-null pointer value
                    Some(ptr::read(EMPTY as *mut T))
                } else {
                    let old = self.ptr;
                    self.ptr = self.ptr.offset(1);

                    Some(ptr::read(old))
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let diff = (self.end as usize) - (self.ptr as usize);
        let size = mem::size_of::<T>();
        let exact = diff /
                    (if size == 0 {
                         1
                     } else {
                         size
                     });
        (exact, Some(exact))
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }
}
