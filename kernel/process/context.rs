//! A module for saving a process's context
//!
//! Each process needs a kernel context and user context
//!
//! The user context is saved to its kernel stack when we
//! switch in to kernel mode. The kernel context is saved
//! to the process struct when we context switch.

use super::CURRENT_PROCESS;

/// A struct representing the contents of the general purpose registers
/// as produced by the `pusha` instruction.
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct KContext {
    pub edi: usize,
    pub esi: usize,
    pub ebp: usize,
    pub esp: usize,
    pub ebx: usize,
    pub edx: usize,
    pub ecx: usize,
    pub eax: usize,
}

impl KContext {
    /// Create a new empty context struct
    pub fn new() -> KContext {
        KContext {
            eax: 0, // These values will never be used, but they are useful for debugging
            ecx: 1,
            edx: 2,
            ebx: 3,
            esp: 4,
            ebp: 5,
            esi: 6,
            edi: 7,
        }
    }
}

/// Save the `KContext` of the current process to its process struct
#[no_mangle]
pub fn store_kcontext(context_ptr: *mut KContext) {
    if context_ptr.is_null() {
        panic!("null context ptr!");
    }

    // Save context to current process
    unsafe {
        if !CURRENT_PROCESS.is_null() {
            (*CURRENT_PROCESS).kcontext = *context_ptr;
        }
    }
}
