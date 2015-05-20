#![allow(dead_code)]

use super::{LinkedList};
use core::option::{Option};

pub struct Queue<T> {
    size: usize,
    data: LinkedList<T>,
}

impl <T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {size: 0, data: LinkedList::new()}
    }

    pub fn push(&mut self, val: T) {
        self.size += 1;
        self.data.push_back(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.size -= 1;
        self.data.pop_front()
    }

    pub fn peek_tail<'a>(&'a self) -> Option<&'a T> {
        self.data.back()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
