// Each process needs a kernel context and user context
//
// The user context is saved to its kernel stack when we
// switch in to kernel mode. The kernel context is saved
// to the process struct when we context switch.

use core::option::Option::{self, Some, None};

use super::current;

#[derive(Clone, Copy)]
pub struct KContext {
    pub eax: usize,
    pub ecx: usize,
    pub edx: usize,
    pub ebx: usize,
    pub esp: usize,
    pub ebp: usize,
    pub esi: usize,
    pub edi: usize,
}

impl KContext {
    pub fn new() -> KContext {
        KContext {
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
        }
    }
}

// save the KContext of the current process to its process struct
#[no_mangle]
pub fn save_context (context_ptr: *mut KContext) {

    if context_ptr == (0 as *mut KContext) {
        panic!("null context ptr!");
    }

    let context = unsafe { *context_ptr };

    match current::current_mut() {
        Some(current_proc) => { current_proc.kcontext = context }
        None => { }
    }
}
