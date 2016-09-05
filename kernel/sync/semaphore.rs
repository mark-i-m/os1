//! A semaphore implementation based on the rust Mutex<T> type

use alloc::boxed::Box;

use core::sync::atomic::{AtomicIsize, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use super::super::interrupts::{on, off};
use super::super::process::{ready_queue, proc_yield, ProcessQueue};

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
    // Use UnsafeCells so that it the semaphore can be used immutably
    count: UnsafeCell<AtomicIsize>,
    queue: UnsafeCell<ProcessQueue>,
}

/// RAII SemaphoreGuard
pub struct SemaphoreGuard<'semaphore, T: 'semaphore> {
    semaphore: &'semaphore StaticSemaphore,
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
    pub fn down<'semaphore>(&'semaphore self) -> SemaphoreGuard<'semaphore, T> {
        // acquire semaphore
        self.inner.down();
        SemaphoreGuard::new(&*self.inner, &self.data)
    }
}

impl<T> Drop for Semaphore<T> {
    /// When the semaphore goes out of scope, it destroys the contents
    fn drop(&mut self) {
        self.inner.destroy();
    }
}

impl StaticSemaphore {
    /// Create a new `StaticSemaphore` with the given count
    pub const fn new(i: isize) -> StaticSemaphore {
        StaticSemaphore {
            count: UnsafeCell::new(AtomicIsize::new(i)),
            queue: UnsafeCell::new(ProcessQueue::new()),
        }
    }

    /// Acquire
    pub fn down(&self) {
        off();
        unsafe {
            if (*self.count.get()).fetch_sub(1, Ordering::AcqRel) <= 0 {
                (*self.count.get()).fetch_add(1, Ordering::AcqRel);
                // block
                proc_yield(Some(&mut *self.queue.get()));
            }
        }
        on();
    }

    /// Release
    pub fn up(&self) {
        off();
        unsafe {
            if let Some(next) = (*self.queue.get()).pop_front() {
                ready_queue::make_ready(next);
            } else {
                (*self.count.get()).fetch_add(1, Ordering::AcqRel);
            }
        }
        on();
    }

    /// Clean up.
    /// Cannot implement Drop here because we want to be able
    /// to create a static semaphore.
    pub fn destroy(&self) {
        unsafe {
            if (*self.queue.get()).len() > 0 {
                // TODO: intead, just kill the processes or do zombie detection
                panic!("Semaphore destroyed with processes waiting!");
            }
        }
    }
}

impl<'semaphore, T> SemaphoreGuard<'semaphore, T> {
    /// Create a guard referring to the given semaphore and data
    fn new(semaphore: &'semaphore StaticSemaphore,
           data: &'semaphore UnsafeCell<T>)
           -> SemaphoreGuard<'semaphore, T> {
        SemaphoreGuard {
            semaphore: semaphore,
            data: data,
        }
    }

    pub fn up(&self) {
        self.semaphore.up();
    }
}

impl<'semaphore, T> Drop for SemaphoreGuard<'semaphore, T> {
    /// `SemaphoreGuard` is RAII, so dropping the guard calls up
    fn drop(&mut self) {
        self.semaphore.up();
    }
}

impl<'semaphore, T> Deref for SemaphoreGuard<'semaphore, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data.get() }
    }
}

impl<'semaphore, T> DerefMut for SemaphoreGuard<'semaphore, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}
