//! A module for inter-process communication
//!
//! In os1, IPC is accomplished primarily through typed
//! shared data structures. A page is shared by one
//! process and accepted by the other.
//!
//! This module provides some useful primitives and
//! abstractions for IPC.

/// A thread-safe abstraction for managing shared memory
///
/// A `SharedMemAllocator` is constructed by a process
/// that wants to share memory with another process. The
/// page-aligned address of the memory the process wants
/// to share is given to the constructor, which will share
/// the memory and return a handle to it.
///
/// The handle can then be used to allocate some of the memory
/// and return parts of it.
pub struct SharedMemAllocator;
