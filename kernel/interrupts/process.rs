// Submodule for cleanly turning interrupts on and off

use core::option::Option::{Some, None};

use super::super::process::current;

use super::super::machine::{cli, sti};

// Turn interrupts on
// There must be a current process to enable interrupts
pub fn on() {
    let me = current::current_mut();

    match me {
        Some(p) => {
            if p.disable_cnt == 0 {
                panic!("Interrupts are already on!");
            }

            p.disable_cnt -= 1;
            if p.disable_cnt == 1 {
                unsafe { sti(); }
            }
        }
        None => panic!("Enable interrupts with no current process!"),
    }
}

// Turn interrupts off
pub fn off() {
    unsafe { cli(); }

    let me = current::current_mut();
    match me {
        Some(p) => p.disable_cnt += 1,
        None => { }
    }
}

// Helper methods for use in IRQ

// Bookkeeping to start an irq
// This should never happen if there is no current process
pub fn start_irq() {
    let me = current::current_mut();
    match me {
        Some(p) => {
            if p.disable_cnt != 0 {
                panic!("start_irq with interrupts disabled!");
            }
            p.disable_cnt = 1;
        }
        None => panic!("start_irq with no current process!"),
    }
}

// Bookkeeping to end an irq
pub fn end_irq() {
    let me = current::current_mut();
    match me {
        Some(p) => {
            if p.disable_cnt != 1 {
                panic!("end_irq with disable_cnt = {} > 1", p.disable_cnt);
            }
            p.disable_cnt = 0;
        }
        None => { }
    }
}
