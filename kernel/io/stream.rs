//! The abstraction of a stream of data

/// A trait for input streams, streams that can be used to get input.
pub trait InputStream {
    /// The type returned by `get`
    type Output;

    /// Get the next `T` from the stream
    fn get(&mut self) -> Self::Output;
}

/// A trait for output streams, streams that can be used to store output.
pub trait OutputStream<T> {
    /// Put the given `T` in the stream
    fn put(&mut self, t: T);
}
