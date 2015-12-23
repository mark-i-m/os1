//! A module for the process table, a table mapping each PID to a process struct

use alloc::boxed::Box;

use super::Process;

/// The process table for the kernel
static mut PROCESS_TABLE: ProcessTable = ProcessTable::new();

/// A process table implemented as an indexable deque
pub struct ProcessTable {
    pt_head: *mut ProcessTableNode,
    pt_tail: *mut ProcessTableNode,

    first_pid: usize,

    capacity: usize,
    size: usize,
}

/// A single node in the `ProcessTable`, holding mappings
/// for 20 processes
struct ProcessTableNode([*mut Process; 20], *mut ProcessTableNode);

impl ProcessTable {
    const fn new() -> ProcessTable {
        ProcessTable {
            pt_head: 0 as *mut ProcessTableNode,
            pt_tail: 0 as *mut ProcessTableNode,

            first_pid: 0,

            capacity: 20,
            size: 0,
        }
    }

    /// Add the new process to the end of the Table. Since processes
    /// are created with sequential PIDs, we already know the PID.
    pub fn push(&mut self, p: *mut Process) {
        // TODO
    }

    /// mark PID `pid` as dead
    pub fn die(&mut self, pid: usize) {
        // TODO
    }
}

impl ProcessTableNode {
    fn new() -> *mut ProcessTableNode {
        Box::into_raw(box ProcessTableNode([0 as *mut Process; 20], 0 as *mut ProcessTableNode))
    }
}
