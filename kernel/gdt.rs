// a module for gdt stuff

extern "C" {
    static gdt: [GDTDescr; 6];

    //pub fn load_gdt(size: u16, offset: &u32);
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct GDTDescr{
    f0: u32,
    f1: u32,
}
