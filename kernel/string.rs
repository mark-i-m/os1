//! A simple String implementation

use core::fmt;
use core::ops::{Deref};
use core::str;

use vec::Vec;

/// A growable UTF-8 string
pub struct String {
    vec: Vec<u8>,
}

impl String {
    /// Return a new empty string: ""
    pub fn new() -> String {
        String {
            vec: Vec::new(),
        }
    }

    /// Push a character to the end
    pub fn push(&mut self, c: char) {
        self.vec.push(c as u8)
    }

    /// Pop and return the last character
    pub fn pop(&mut self) -> Option<char> {
        self.vec.pop().map(|ou8| ou8 as char)
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl Deref for String {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.vec) }
    }
}

impl fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl Clone for String {
    fn clone(&self) -> Self {
        String { vec: self.vec.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.vec.clone_from(&source.vec);
    }
}
