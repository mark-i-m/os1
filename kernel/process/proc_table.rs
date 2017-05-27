//! A module for the process table, a table mapping each PID to a process struct

use alloc::boxed::Box;

use core::ops::{Index, IndexMut};

use sync::StaticSemaphore;
use super::Process;

/// The process table for the kernel
pub static mut PROCESS_TABLE: ProcessTable = ProcessTable::new();

/// A lock for the `PROCESS_TABLE`
static mut TABLE_LOCK: StaticSemaphore = StaticSemaphore::new(1);

/// Number of PIDs mapped by a single `ProcessTableNode`
const NODE_SIZE: usize = 20;

/// A process table implemented as an indexable deque
pub struct ProcessTable {
    // The head and tail of the table/list
    pt_head: *mut ProcessTableNode,

    first_pid: usize, // The first PID that is actually in the table

    capacity: usize, // The max PID that can be mapped + 1
    size: usize, // The max PID that is mapped + 1
}

/// A single node in the `ProcessTable`, holding mappings
/// for `NODE_SIZE` processes
struct ProcessTableNode {
    map: [*mut Process; NODE_SIZE],
    next: *mut ProcessTableNode,
}

impl ProcessTable {
    const fn new() -> ProcessTable {
        ProcessTable {
            pt_head: 0 as *mut ProcessTableNode,

            first_pid: 0,

            capacity: 0,
            size: 0,
        }
    }

    /// Add the new process to the end of the Table. Since processes
    /// are created with sequential PIDs, we already know the PID.
    pub fn push(&mut self, p: *mut Process) {
        // lock
        unsafe {
            TABLE_LOCK.down();
        }

        // Resize if needed
        if self.capacity == self.size {
            if self.pt_head.is_null() {
                self.pt_head = ProcessTableNode::new();
            } else {
                self.get_last_node().next = ProcessTableNode::new();
            }

            self.capacity += NODE_SIZE;
        }

        // Append to the table
        let last = self.get_last_node();
        last[self.size % NODE_SIZE] = p;
        self.size += 1;

        // unlock
        unsafe {
            TABLE_LOCK.up();
        }
    }

    /// Void the entry in the table. This should be done just before
    /// deallocating the process.
    pub fn remove(&mut self, pid: usize) {
        // lock
        unsafe {
            TABLE_LOCK.down();
        }

        // void the entry
        let node = self.get_node_mut(pid);
        node[pid % NODE_SIZE] = 0 as *mut Process;

        // unlock
        unsafe {
            TABLE_LOCK.up();
        }

        // free as much space as possible
        self.free()
    }

    /// Get the process with the matching PID
    pub fn get(&self, pid: usize) -> Option<*mut Process> {
        // corner case
        if pid < self.first_pid {
            return None;
        }

        // lock
        unsafe {
            TABLE_LOCK.down();
        }

        let node = self.get_node(pid);
        let result = node[pid % NODE_SIZE];

        // unlock
        unsafe {
            TABLE_LOCK.up();
        }

        if result.is_null() { None } else { Some(result) }
    }

    /// Free as many `ProcessTableNode`s as possible
    fn free(&mut self) {
        // lock
        unsafe {
            TABLE_LOCK.down();
        }

        let mut node = self.pt_head;
        unsafe {
            while !(*node).next.is_null() && (*node).is_empty() {
                let next = (*node).next;
                ProcessTableNode::destroy(node);
                node = next;
                self.pt_head = next;
                self.first_pid += NODE_SIZE;
            }
        }

        // unlock
        unsafe {
            TABLE_LOCK.up();
        }
    }

    /// Get the node containing the given PID
    fn get_node(&self, mut pid: usize) -> &'static ProcessTableNode {
        // Corner cases
        if pid >= self.capacity {
            panic!("PID does not exist!");
        }
        if pid < self.first_pid {
            panic!("PID already TERMINATED");
        }

        // find node with pid
        let mut node = self.pt_head;
        let mut p = self.first_pid;

        // Round down to NODE_SIZE
        pid -= pid % NODE_SIZE;

        unsafe {
            while !(*node).next.is_null() && p < pid {
                node = (*node).next;
                p += NODE_SIZE;
            }
            &mut (*node)
        }
    }

    /// Get the node containing the given PID
    fn get_node_mut(&mut self, mut pid: usize) -> &'static mut ProcessTableNode {
        // Corner cases
        if pid >= self.capacity {
            panic!("PID {} does not exist!", pid);
        }
        if pid < self.first_pid {
            panic!("PID {} already TERMINATED", pid);
        }

        // find node with pid
        let mut node = self.pt_head;
        let mut p = self.first_pid;

        // Round down to NODE_SIZE
        pid -= pid % NODE_SIZE;

        unsafe {
            while !(*node).next.is_null() && p < pid {
                node = (*node).next;
                p += NODE_SIZE;
            }
            &mut (*node)
        }
    }

    /// Get the last node in the table
    fn get_last_node(&mut self) -> &'static mut ProcessTableNode {
        let cap = self.capacity - 1;
        self.get_node_mut(cap)
    }
}

impl ProcessTableNode {
    fn new() -> *mut ProcessTableNode {
        Box::into_raw(box ProcessTableNode {
                              map: [0 as *mut Process; 20],
                              next: 0 as *mut ProcessTableNode,
                          })
    }

    fn destroy(n: *mut ProcessTableNode) {
        unsafe {
            Box::from_raw(n);
        }
    }

    fn is_empty(&self) -> bool {
        (0..NODE_SIZE).all(|i| self[i].is_null())
    }
}

impl Index<usize> for ProcessTableNode {
    type Output = *mut Process;

    fn index<'a>(&'a self, index: usize) -> &'a *mut Process {
        &self.map[index]
    }
}

impl IndexMut<usize> for ProcessTableNode {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut *mut Process {
        &mut self.map[index]
    }
}
