//! A module for I/O stuff

pub use self::nbb::NonBlockingBuffer;

pub mod kbd;
pub mod stream;

mod nbb;
