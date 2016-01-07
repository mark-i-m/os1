//! A module for I/O stuff

pub use self::nbb::NonBlockingBuffer;

pub mod stream;
pub mod kbd;
pub mod block;
pub mod ide;

mod nbb;
