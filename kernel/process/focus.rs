//! A module for process focus

use super::super::interrupts::{on, off};
use super::CURRENT_PROCESS;

/// The pid of the currently focused process
static mut FOCUSED_PID: usize = 0;

/// Set focus on the given pid if one is given,
/// or on the CURRENT_PROCESS if None is given.
///
/// This function does no error checking on the
/// state of the process.
pub fn focus(pid: Option<usize>) {
    off();
    unsafe {
        if let Some(pid) = pid {
            FOCUSED_PID = pid;
        } else {
            FOCUSED_PID = (*CURRENT_PROCESS).get_pid();
        }
    }
    on();
}

/// Returns the FOCUSED_PID in a thread-safe way.
///
/// This function does no error checking on the
/// state of the process.
pub fn get_focused() -> usize {
    off();
    let pid = unsafe { FOCUSED_PID };
    on();
    pid
}
