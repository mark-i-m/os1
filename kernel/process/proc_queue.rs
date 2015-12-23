use super::super::process::{Process};

/// An abstraction for any kind of process queue
///
/// This structure should be used whenever queueing up processes. It uses raw
/// pointers because it is often necessary to have multiple pointers to a
/// process with atomic access to the process (as opposed to making a function
/// call to get a pointer to the process).
///
/// It is uses the pointer already present in the `Process` struct to manage
/// chains of processes.
///
/// This is not very Rustic at all :( but it cannot be helped because Rust's
/// abstractions are based on the OS in many cases.
///
/// Because of this, the queue does not do any memory management at all!
pub struct ProcessQueue {
    pub head: *mut Process,
    pub tail: *mut Process,
}

impl ProcessQueue {
    /// Create a new, empty queue. This is a const function, so statics can have
    /// queues.
    pub const fn new() -> ProcessQueue {
        ProcessQueue {
            head: 0 as *mut Process,
            tail: 0 as *mut Process,
        }
    }

    /// Push the given process to the end of the queue. O(1)
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

    /// Pop and return the head of the queue. If this method returns a null
    /// pointer, then the queue is empty. O(1)
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
