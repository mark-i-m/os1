// A module for concurrency control

pub use self::atomic32::Atomic32;

pub use self::semaphore::Semaphore;

mod atomic32;

mod semaphore;
