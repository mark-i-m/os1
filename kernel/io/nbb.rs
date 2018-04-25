//! A Non-blocking buffer implementation

use alloc::raw_vec::RawVec;

use core::ptr;

use interrupts::no_interrupts;

use super::stream::*;

/// A non-blocking circular buffer for use
/// by interrupt handlers
pub struct NonBlockingBuffer {
    // A buffer
    buffer: RawVec<char>,

    // index of the front of the buffer
    front: usize,

    // number of elements in the buffer
    size: usize,
}

impl NonBlockingBuffer {
    pub fn new(cap: usize) -> NonBlockingBuffer {
        NonBlockingBuffer {
            buffer: RawVec::with_capacity(cap),
            front: 0,
            size: 0,
        }
    }
}

impl InputStream for NonBlockingBuffer {
    type Output = Option<char>;

    /// Get the next character in the stream if there is one
    fn get(&mut self) -> Option<char> {
        no_interrupts(|| {
            if self.size > 0 {
                let i = self.front;
                self.front = (self.front + 1) % self.buffer.cap();
                self.size -= 1;
                unsafe { Some(ptr::read(self.buffer.ptr().offset(i as isize))) }
            } else {
                None
            }
        })
    }
}

impl Iterator for NonBlockingBuffer {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.get()
    }
}

impl OutputStream<char> for NonBlockingBuffer {
    /// Put the given character at the end of the buffer.
    ///
    /// If there is no room in the buffer, the character is
    /// dropped.
    fn put(&mut self, c: char) {
        no_interrupts(|| {
            if self.size < self.buffer.cap() {
                let next = (self.front + self.size) % self.buffer.cap();
                unsafe {
                    ptr::write(self.buffer.ptr().offset(next as isize), c);
                }
                self.size += 1;
            }
        })
    }
}
