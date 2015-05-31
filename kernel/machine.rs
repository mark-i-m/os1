#[link(name = "asmcode", repr="static")]

#[allow(improper_ctypes)]
#[allow(dead_code)]
extern "C" {
    // pub fn switchToUser(pc: usize, esp: usize, eax: usize);

    // pub fn vmm_on(cr3: usize);
    // pub fn uint32_t cs32(void *ptr, uint32_t ifval, uint32_t thenval);
    pub fn save_kcontext();
    pub fn context_switch(next_context: super::process::context::KContext, eflags: usize);
    pub fn eflags() -> usize;

    // pub fn pit_do_init(divide: u32);

    pub fn inb(port: u16) -> u8; // if there are problems, change u8 to u32
    pub fn inl(port: u16) -> u32;
    pub fn outb(port: u16, val: u8);

    // pub fn ltr(tr: usize);

    // pub fn pageFaultHandler();
    // pub fn syscallTrap();

    // pub fn getcr0() -> usize;
    // pub fn getcr3() -> usize;
    // pub fn invlpg(pg: u32);

    // pub fn cli();
    // pub fn sti();

    // pub fn irq0();
    // pub fn irq1();
    // pub fn irq2();
    // pub fn irq3();
    // pub fn irq4();
    // pub fn irq5();
    // pub fn irq6();
    // pub fn irq7();
    // pub fn irq8();
    // pub fn irq9();
    // pub fn irq10();
    // pub fn irq11();
    // pub fn irq12();
    // pub fn irq13();
    // pub fn irq14();
    // pub fn irq15();

    //pub fn sys_sigret(uint32_t);
}
