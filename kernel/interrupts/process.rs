//! Module for cleanly turning interrupts on and off

use super::super::machine::{cli, sti};
use super::super::process::CURRENT_PROCESS;

/// Turn interrupts on
pub fn on() {
    unsafe {
        // Meaningless to enable interrupts w/o current proc
        if !CURRENT_PROCESS.is_null() {
            if (*CURRENT_PROCESS).disable_cnt == 0 {
                panic!("Interrupts are already on!");
            }

            (*CURRENT_PROCESS).disable_cnt -= 1;
            if (*CURRENT_PROCESS).disable_cnt == 0 {
                sti();
            }
        }
    }
}

/// Turn interrupts off
pub fn off() {
    unsafe {
        cli();

        if !CURRENT_PROCESS.is_null() {
            (*CURRENT_PROCESS).disable_cnt += 1;
        }
    }
}

/// Helper method for use in IRQ
/// Bookkeeping to start an irq.
/// This should never happen if there is no current process.
pub fn start_irq() {
    unsafe {
        if !CURRENT_PROCESS.is_null() {
            if (*CURRENT_PROCESS).disable_cnt != 0 {
                panic!("start_irq with interrupts disabled!");
            }
            (*CURRENT_PROCESS).disable_cnt = 1;
        } else {
            panic!("start_irq with no current process!");
        }
    }
}

/// Helper method for use in IRQ
/// Bookkeeping to end an irq
pub fn end_irq() {
    unsafe {
        if !CURRENT_PROCESS.is_null() {
            if (*CURRENT_PROCESS).disable_cnt != 1 {
                panic!(
                    "end_irq with disable_cnt = {} > 1",
                    (*CURRENT_PROCESS).disable_cnt
                );
            }
            (*CURRENT_PROCESS).disable_cnt = 0;
            // bootlog!("{:?} [End irq]\n", *CURRENT_PROCESS);
        }
    }
}
