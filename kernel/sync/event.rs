//! A module containing an event implementation

use super::StaticSemaphore;

/// An Event data structure.
///
/// Processes calling `wait` will block until a process
/// calls `notify`.
pub struct Event {
    status: StaticSemaphore,
}

impl Event {
    /// Creates a new `Event`
    pub const fn new() -> Event {
        Event {
            status: StaticSemaphore::new(0),
        }
    }

    /// Block this process until `notify` is called
    pub fn wait(&mut self) {
        self.status.down();
        self.status.up();
    }

    /// Wake up all processes that called `wait` on this event
    pub fn notify(&mut self) {
        self.status.up();
    }

    /// Reset the event before more processes can `wait`
    pub fn reset(&mut self) {
        self.status.down();
    }
}
