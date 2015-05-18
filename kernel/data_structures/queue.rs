#![allow(dead_code)]

use alloc::boxed::{Box};
use core::option::Option::{self, Some, None};

pub struct Queue<T> {
    size: usize,
    data: Option<Box<QueueNode<T>>>,
}

struct QueueNode<T> {
    data: Box<T>,
    next: Option<Box<QueueNode<T>>>,
}

impl <T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {size: 0, data: None}
    }

    pub fn push(&mut self, val: T) {
        //TODO
    }

    pub fn pop(&mut self) -> Option<T> {
        None // TODO
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
