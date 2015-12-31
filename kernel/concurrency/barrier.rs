//! A module containing a barrier implementation

use super::super::interrupts::{on,off};
use super::Event;

/// A Barrier implementation.
///
/// `N-1` processes wait until the `N`th process
/// arrives at the barrier. `N` is defined when
/// the Barrier is created. The Barrier is reusable.
pub struct Barrier {
    count: usize,
    n: usize,
    event: Event,
}

impl Barrier {
    /// Create a new barrier for `n` processes
    pub const fn new(n: usize) -> Barrier {
        Barrier {
            count: 0,
            n: n,
            event: Event::new(),
        }
    }

    /// This process reaches the barrier
    pub fn reach(&mut self) {
        off();
        if self.count == self.n - 1 {
            self.count = 0;
            self.event.notify();
        } else {
            self.count += 1;
            self.event.wait();
        }
        on();
    }
}
