//! A module for handling system calls

use interrupts::add_trap_handler;
use machine::syscall_trap;
use process;

/// Initialize the system call subsystem.
/// Use IRQ 100 as the system call trap.
pub fn init() {
    add_trap_handler(100, syscall_trap, 3);
}

/// The system call handler
#[no_mangle]
#[inline(never)]
pub unsafe fn syscall_handler(context: *mut usize, syscall_num: usize, a0: usize, a1: usize) {
    match syscall_num {
        0 => { // exit
            process::exit(a0);
        }
        _ => {
            panic!("system call #{}\n", syscall_num);
        }
    }
}
