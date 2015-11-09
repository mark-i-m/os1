// The programmable interrupt controller

use core::option::Option::None;

use super::super::machine::*;

use super::idt::add_interrupt_handler;

const C1: u16 = 0x20;          /* command port for PIC1 */
const D1: u16 = 0x21;          /* data port for PIC1 */
const C2: u16 = 0xA0;          /* command port for PIC2 */
const D2: u16 = 0xA1;          /* data port for PIC2 */

const FIRST_IDT: u8 = 0x30;

pub fn init() {
    unsafe {
        /* ICW1 */
        outb(C1,0x11);        /* init with ICW4, not single */
        outb(C2,0x11);        /* init with ICW4, not single */

        /* ICW2 */
        outb(D1,FIRST_IDT);    /* IDT index for IRQ0 */
        outb(D2,FIRST_IDT+8);  /* IDT index for IRQ8 */

        /* ICW3 */
        outb(D1, 1 << 2);     /* tells master that the slave is at IRQ2 */
        outb(D2, 2);          /* tells salve that it's connected at IRQ2 */

        /* ICW4 */
        outb(D1, 1);          /* 8086 mode */
        outb(D2, 1);          /* 8086 mode */

        /* enable all */
        outb(D1,0);
        outb(D2,0);

        add_interrupt_handler(FIRST_IDT + 0 , irq0);
        add_interrupt_handler(FIRST_IDT + 1 , irq1);
        add_interrupt_handler(FIRST_IDT + 2 , irq2);
        add_interrupt_handler(FIRST_IDT + 3 , irq3);
        add_interrupt_handler(FIRST_IDT + 4 , irq4);
        add_interrupt_handler(FIRST_IDT + 5 , irq5);
        add_interrupt_handler(FIRST_IDT + 6 , irq6);
        add_interrupt_handler(FIRST_IDT + 7 , irq7);
        add_interrupt_handler(FIRST_IDT + 8 , irq8);
        add_interrupt_handler(FIRST_IDT + 9 , irq9);
        add_interrupt_handler(FIRST_IDT + 10, irq10);
        add_interrupt_handler(FIRST_IDT + 11, irq11);
        add_interrupt_handler(FIRST_IDT + 12, irq12);
        add_interrupt_handler(FIRST_IDT + 13, irq13);
        add_interrupt_handler(FIRST_IDT + 14, irq14);
        add_interrupt_handler(FIRST_IDT + 15, irq15);
    }

    printf!("PIC inited\n");
}

// End of interrupt: send the next irq, but interrupts still disabled
fn pic_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            /* let PIC2 know */
            outb(C2,0x20);
        }
        /* we always let PIC1 know because PIC2 is routed though PIC1 */
        outb(C1,0x20);
    }
}

// This is the code for the interrupt handler
#[no_mangle]
pub fn pic_irq(irq: usize) {
    // bookkeeping
    super::process::start_irq();

    // execute handler
    match irq {
        0   => super::pit::handler(),
        1   => { } // keyboard
        13  => { } // Processor, FPU
        _   => {
            unsafe { cli(); }
            panic!("interrupt {}\n",irq)
        },
    }

    // the PIC can deliver the next interrupt, but interrupts are still disabled
    pic_eoi(irq as u8);

    // only do this for pit interrupts
    if irq == 0 {
        unsafe {
            super::super::process::proc_yield(None);
        }
    }

    // bookkeeping
    super::process::end_irq();
}
