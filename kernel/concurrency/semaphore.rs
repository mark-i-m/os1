//! A semaphore implementation based on the rust Mutex<T> type

use core::sync::atomic::{AtomicIsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use alloc::boxed::Box;

use super::super::process::{ready_queue, proc_yield, ProcessQueue};

use super::super::interrupts::{on,off};

/// `Semaphore` is a much more Rustic semaphore. It returns an RAII
/// `SemaphoreGuard`, which automatically calls "up" when it goes out of
/// scope. This semaphore takes ownership of the data it is guarding, so that
/// Rust ownership and lifetime semantics can be used to guarantee safety of
/// the resource.
///
/// ```
///  // wrap resource in a semaphore with initial count 3
///  let s = Semaphore::new(resource, 3);
///
///  {
///      // acquire
///      let guard = s.clone();
///  }
///  // guard dies => release
/// ```
pub struct Semaphore<T> {
    // box the inner semaphore so it has a constant address
    // and can be safely moved
    inner: Box<StaticSemaphore>,
    data: UnsafeCell<T>,
}

/// `StaticSemaphore` is a semaphore implementation that can be used in
/// statics. It has a const constructor.
pub struct StaticSemaphore {
    count: AtomicIsize,
    queue: ProcessQueue,
}

/// RAII SemaphoreGuard
pub struct SemaphoreGuard<'semaphore, T: 'semaphore> {
    semaphore: &'semaphore mut StaticSemaphore,
    data: &'semaphore UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Semaphore<T> {}
unsafe impl<T: Send> Sync for Semaphore<T> {}

impl<T> Semaphore<T> {
    /// Create a new semaphore with the the given count guarding the given value
    pub fn new(val: T, i: isize) -> Semaphore<T> {
        Semaphore {
            inner: box StaticSemaphore::new(i),
            data: UnsafeCell::new(val),
        }
    }

    /// Acquire.
    /// returns an RAII guard, so no need for up()
    pub fn down<'semaphore>(&'semaphore mut self) -> SemaphoreGuard<'semaphore, T> {
        // acquire semaphore
        self.inner.down();
        SemaphoreGuard::new(&mut *self.inner, &self.data)
    }
}

impl<T> Drop for Semaphore<T> {
    /// When the semaphore goes out of scope, it destroys the contents
    fn drop(&mut self) {
        self.inner.destroy();
    }
}

// TODO: make StaticSemaphore use an unsafe cell, so it can be used immutably
impl StaticSemaphore {
    /// Create a new `StaticSemaphore` with the given count
    pub const fn new(i: isize) -> StaticSemaphore {
        StaticSemaphore {
            count: AtomicIsize::new(i),
            queue: ProcessQueue::new(),
        }
    }

    /// Acquire
    pub fn down(&mut self) {
        off();
        if self.count.fetch_sub(1, Ordering::AcqRel) <= 0 {
            self.count.fetch_add(1, Ordering::AcqRel);
            // block
            proc_yield(Some(&mut self.queue));
        }
        on();
    }

    /// Release
    pub fn up(&mut self) {
        off();
        if let Some(next) = self.queue.pop_front() {
            ready_queue::make_ready(next);
        } else {
            self.count.fetch_add(1,Ordering::AcqRel);
        }
        on();
    }

    /// Clean up.
    /// Cannot implement Drop here because we want to be able
    /// to create a static semaphore.
    pub fn destroy(&mut self) {
        if self.queue.len() > 0 {
            // TODO: intead, just kill the processes or do zombie detection
            panic!("Semaphore destroyed with processes waiting!");
        }
    }
}

impl<'semaphore, T> SemaphoreGuard<'semaphore, T> {
    /// Create a guard referring to the given semaphore and data
    fn new(semaphore: &'semaphore mut StaticSemaphore,
               data: &'semaphore UnsafeCell<T>) -> SemaphoreGuard<'semaphore, T>{
        SemaphoreGuard {
            semaphore: semaphore,
            data: data,
        }
    }
}

impl<'semaphore, T> Drop for SemaphoreGuard<'semaphore, T> {
    /// `SemaphoreGuard` is RAII, so dropping the guard calls up
    fn drop(&mut self) {
        self.semaphore.up();
    }
}

/// Implement Deref and DerefMut to get deref coersions
impl<'semaphore, T> Deref for SemaphoreGuard<'semaphore, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data.get() }
    }
}

/// Implement Deref and DerefMut to get deref coersions
impl<'semaphore, T> DerefMut for SemaphoreGuard<'semaphore, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

