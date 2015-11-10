// An abstraction for any kind of process queue

#![allow(dead_code)]

use super::super::process::{Process};

pub struct ProcessQueue {
    pub head: *mut Process,
    pub tail: *mut Process,
}

impl ProcessQueue {
    pub const fn new() -> ProcessQueue {
        ProcessQueue {
            head: 0 as *mut Process,
            tail: 0 as *mut Process,
        }
    }

    pub fn push_tail(&mut self, process: *mut Process) {
        unsafe {
            // Add process to end of queue
            if !self.tail.is_null() {
                (*self.tail).next_proc = process;
            }
            self.tail = process;

            // Handle case where tail is head
            if self.head.is_null() {
                self.head = process;
            }
        }
    }

    pub fn pop_head(&mut self) -> *mut Process {
        unsafe {
            // No ready processes
            if self.head.is_null() {
                return self.head;
            }

            let next = self.head;

            self.head = (*self.head).next_proc;

            // Dequeued last element
            if self.head.is_null() {
                self.tail = self.head;
            }

            // Remove link to next process
            (*next).next_proc = 0 as *mut Process;

            next
        }
    }
}
