//! This is the Rust-side handle for assembly code. Most of the FFI handles are
//! in this module.

#[link(name = "asmcode", repr = "static")]
use process::ProcessQueue;

#[allow(improper_ctypes)]
#[allow(dead_code)]
extern "C" {
    /// a wrapper around inb
    pub fn inb(port: u16) -> u8;

    /// a wrapper around inl
    pub fn inl(port: u16) -> u32;

    /// a wrapper around outb
    pub fn outb(port: u16, val: u8);

    /// a wrapper around outl
    pub fn outl(port: u16, val: u32);

    /// a wrapper around ltr
    pub fn ltr(tr: usize);

    /// Initialize the PIT with the given divide
    pub fn pit_do_init(divide: usize);

    /// Turn on VM and load cr3 with the given value
    pub fn vmm_on(cr3: usize);
    // pub fn getcr0() -> usize;
    // pub fn getcr3() -> usize;

    /// Flush the page from the TLB
    pub fn invlpg(pg: usize);

    /// a wrapper around cli
    pub fn cli();

    /// a wrapper around sti
    pub fn sti();

    /// A handler for IRQ 0
    pub fn irq0();

    /// A handler for IRQ 1
    pub fn irq1();

    /// A handler for IRQ 2
    pub fn irq2();

    /// A handler for IRQ 3
    pub fn irq3();

    /// A handler for IRQ 4
    pub fn irq4();

    /// A handler for IRQ 5
    pub fn irq5();

    /// A handler for IRQ 6
    pub fn irq6();

    /// A handler for IRQ 7
    pub fn irq7();

    /// A handler for IRQ 8
    pub fn irq8();

    /// A handler for IRQ 9
    pub fn irq9();

    /// A handler for IRQ 10
    pub fn irq10();

    /// A handler for IRQ 11
    pub fn irq11();

    /// A handler for IRQ 12
    pub fn irq12();

    /// A handler for IRQ 13
    pub fn irq13();

    /// A handler for IRQ 14
    pub fn irq14();

    /// A handler for IRQ 15
    pub fn irq15();

    /// An unsafe proc_yield handle that saves the context of the current
    /// process before switching. *Do not* call this function directly! Instead,
    /// use `process::proc_yield`, which is a wrapper around this function.
    ///
    /// NOTE: Interrupts should disable before calling this function.
    #[inline(never)]
    pub fn proc_yield<'a>(q: Option<&'a mut ProcessQueue>);

    /// Do a context switch to `next_context` with the eflags register set to
    /// `eflags`. This function is called by `process::proc_yield`
    pub fn context_switch(next_context: super::process::context::KContext, eflags: usize);

    /// Returns the value of eflags
    pub fn eflags() -> usize;

    /// Switch to usermode with the given PC, stack pointer, and %eax
    pub fn switch_to_user(pc: usize, esp: usize, eax: usize);

    /// The assembly handle for the page fault handler. This function calls
    /// `vmm_page_fault`.
    pub fn page_fault_handler();

    /// The assembly handle for the system call trap handler. This function calls
    /// `syscall_handler`
    pub fn syscall_trap();

// pub fn sys_sigret(uint32_t);
}
