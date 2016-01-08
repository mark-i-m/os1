//! A module for trait definitions that file system
//! objects must implement. Essentially, this is the
//! virtual file system...

use core::fmt::Display;

/// A trait that all file systems must implement
pub trait FileSystem {
    /// A type representing files
    type File: FSFile;

    /// A type representing file paths
    type Path: FSPath;

    /// A type representing file system errors
    type Error: FSError;

    /// Read the file system from the given device
    /// and return a handle to it
    fn new(device: usize) -> Self;

    /// Open the file indicated by the `path` and
    /// return a handle to it.
    fn open(&mut self, path: Self::Path) -> Result<Self::File, Self::Error>;

    /// Close the given file.
    fn close(&mut self, file: Self::File) -> Option<Self::Error>;
}

/// A trait that all files must implement
pub trait FSFile {
    /// A type representing file system errors
    type Error: FSError;

    /// Get the size of this buffer in bytes
    fn size(&self) -> usize;

    // TODO: something with an internal offset
}

/// A trait that all path types must implement
pub trait FSPath {
    /// A type representing file system errors
    type Error: FSError;

    /// Split the path into its components
    fn split(&self) -> [&str];
}

/// A trait for representing file system errors
/// based on the libstd Error trait.
pub trait FSError: Display {
    fn description(&self) -> &str;

    fn cause(&self) -> Option<&FSError> { None }
}
