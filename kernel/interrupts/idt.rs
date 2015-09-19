// This module contains idt stuff

extern "C" {
    static kernelCodeSeg: u16;
    static mut idt: [IDTDescr; 256];
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct TableDescriptor {
    size: u16,
    location: u32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct IDTDescr{
   offset_1: u16,   // offset bits 0..15
   selector: u16,   // a code segment selector in GDT or LDT
   zero: u8,        // unused, set to 0
   type_attr: u8,   // type and attributes, see below
   offset_2: u16,   // offset bits 16..31
}

impl IDTDescr {
    pub fn new() -> IDTDescr {
        IDTDescr {
            offset_1: 0,
            offset_2: 0,
            zero:     0,
            type_attr:0,
            selector: 0,
        }
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset_1 = (offset & 0xFFFF) as u16;
        self.offset_2 = ((offset >> 16) & 0xFFFF) as u16;
    }

    pub fn set_type_attr(&mut self, present: bool, dpl: u8, storage_seg: bool, gate_type: u8) {
        if dpl > 3 { panic!("dpl > 3"); }
        if gate_type > 1<<4 { panic!("gate_type > 15"); }

        self.type_attr = ((present as u8) << 7) | (dpl << 5) | ((storage_seg as u8) << 4) | gate_type;
    }

    pub fn set_selector(&mut self, selector: u16) {
        self.selector = selector;
    }
}

pub fn add_interrupt_handler(irq: u8, handler: unsafe extern "C" fn()) {
    let idx = irq as usize;
    unsafe {
        idt[idx] = IDTDescr::new();
        idt[idx].set_offset(handler as u32);
        idt[idx].set_selector(kernelCodeSeg);
        idt[idx].set_type_attr(true, 0, false, 0xE);
    }
}
