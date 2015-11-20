// A semaphore implementation based on the rust Mutex<T> type

use core::sync::atomic::{AtomicIsize, Ordering};
use core::cell::UnsafeCell;
use core::option::Option::Some;

use alloc::boxed::Box;

use super::super::ProcessQueue;

use super::super::super::process::{ready_queue, proc_yield};

use super::super::super::interrupts::{on,off};

pub struct Semaphore<T> {
    // box the inner semaphore so it has a constant address
    // and can be safely moved
    inner: Box<StaticSemaphore>,
    data: UnsafeCell<T>,
}

// static semaphore that is not RAII
pub struct StaticSemaphore {
    count: AtomicIsize,
    queue: ProcessQueue,
}

// RAII native SemaphoreGuard
pub struct SemaphoreGuard<'a, T: 'a> {
    semaphore: &'a Semaphore<T>,
}


unsafe impl<T: Send> Send for Semaphore<T> {}
unsafe impl<T: Send> Sync for Semaphore<T> {}

impl<T> Semaphore<T> {
    pub fn new(val: T, i: isize) -> Semaphore<T> {
        Semaphore {
            inner: box StaticSemaphore::new(i),
            data: UnsafeCell::new(val),
        }
    }

    // TODO
}

impl<T> Drop for Semaphore<T> {
    fn drop(&mut self) {
        self.inner.destroy();
    }
}


impl StaticSemaphore {
    pub const fn new(i: isize) -> StaticSemaphore {
        StaticSemaphore {
            count: AtomicIsize::new(i),
            queue: ProcessQueue::new(),
        }
    }

    pub fn down(&mut self) {
        off();
        if self.count.fetch_sub(1, Ordering::AcqRel) <= 0 {
            self.count.fetch_add(1, Ordering::AcqRel);
            // block
            proc_yield(Some(&mut self.queue));
        }
        on();
    }

    pub fn up(&mut self) {
        off();
        let next = self.queue.pop_head();
        if next.is_null() {
            self.count.fetch_add(1,Ordering::AcqRel);
        } else {
            ready_queue::make_ready(next);
        }
        on();
    }

    // clean up
    // cannot implement Drop here because we want to be able
    // to create a static semaphore
    pub fn destroy(&mut self) {
        if !self.queue.pop_head().is_null() {
            // TODO: intead, just kill the processes or do zombie detection
            panic!("Semaphore destroyed with processes waiting!");
        }
    }
}








impl<'a, T> SemaphoreGuard<'a, T> {

}
