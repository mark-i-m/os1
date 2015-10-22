use super::Process;

// current process
pub static mut CURRENT_PROCESS: *mut Process = 0 as *mut Process;

///////////////////////////////////////////////////////////////
// DO NOT USE on/off here b/c there will be circular reference.
// Need to use cli/sti here, but be careful to leave interrupt
// flag as we found it.
///////////////////////////////////////////////////////////////

// Init the current process
pub fn init() {
}
