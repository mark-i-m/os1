// Each process needs a kernel context and user context
//
// The user context is saved to its kernel stack when we
// switch in to kernel mode. The kernel context is saved
// to the process struct when we context switch.

use core::option::Option::{self, Some, None};

use super::current;

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
    pub fn new() -> KContext {
        KContext {
            eax: 0, //These values will never be used, but they are useful for debugging
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

// save the KContext of the current process to its process struct
#[no_mangle]
pub fn store_kcontext (context_ptr: *mut KContext) {

    if context_ptr == (0 as *mut KContext) {
        panic!("null context ptr!");
    }

    let context = unsafe { *context_ptr };

    match current::current_mut() {
        Some(current_proc) => { current_proc.kcontext = context }
        None => { }
    }
}
