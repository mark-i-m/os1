// Process address spaces

use core::ops::{Index, IndexMut};

use core::intrinsics::transmute;

use super::physmem::Frame;

use super::super::machine::{invlpg, vmm_on, page_fault_handler};

use super::super::interrupts::{add_trap_handler, on, off};

use super::super::process::CURRENT_PROCESS;

// shared PDEs direct mapping the first 8MiB
static mut PDE0: PagingEntry = PagingEntry::new();
static mut PDE1: PagingEntry = PagingEntry::new();

// is VM on
static mut VMM_ON: bool = false;

// The address space of a single process
pub struct AddressSpace {
    // paddr of this addr space's PD
    page_dir: usize,

    // the index of the first un-kmapped page
    kmap_index: u8,
}

// A single entry in a page directory or table
// 31                 12  9       0
// [                    000 flags  ]
#[derive(Copy, Clone)]
#[repr(C,packed)]
struct PagingEntry {
    entry: usize,
}

#[repr(C,packed)]
struct VMTable {
    entries: [PagingEntry; 1024],
}

impl AddressSpace {
    pub fn new() -> AddressSpace {
        // allocate a frame
        let (pd, pd_paddr) = VMTable::new();

        // copy the first two entries of the pd to direct map
        // the first 8MiB
        pd[0] = unsafe {PDE0};
        pd[1] = unsafe {PDE1};

        // point the third PDE at the PD to map all page tables
        // so PD is at vaddr 0x802000
        pd[2].set_present(true); // present
        pd[2].set_read_write(true); // read/write
        pd[2].set_privelege_level(false); // kernel only
        pd[2].set_caching(true); // write-through
        pd[2].set_address(pd_paddr); // PD paddr

        let a = AddressSpace {
            page_dir: pd_paddr,
            kmap_index: 0,
        };

        // if VM on
        // clean up after kmap
        // TODO: kunmap? we cannot do the below because this will
        // also free the mapped frame, which we don't want
        //unsafe {
        //    if !CURRENT_PROCESS.is_null() && VMM_ON {
        //        (*CURRENT_PROCESS).addr_space.unmap(pd as *mut VMTable as usize);
        //    }
        //}

        a
    }

    // map the va to the pa in the address space
    // NOTE: should only be called on the current address space
    // because it assumes that the PD is at 0x802000
    fn map(&mut self, phys: usize, virt: usize) {
        off();

        //unsafe {
        //    bootlog!("{:?} [map {:x} -> {:x}]\n", *CURRENT_PROCESS, virt, phys);
        //}

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let mut pd = unsafe {&mut *(0x802000 as *mut VMTable)};

        // if pd entry is not already there
        let pde = &mut pd[pde_index];
        if !pde.is_flag(0) { // present bit
            // no pd entry yet => create one

            // set pde
            pde.set_read_write(true); // read/write
            pde.set_privelege_level(false); // kernel only
            pde.set_caching(false); // write-back
            pde.set_address(Frame::alloc()); // alloc a new frame
            pde.set_present(true); // present

            // clear the pt
            let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};
            for p in 0..1024 {
                pt[p] = PagingEntry::new();
                unsafe { invlpg((pde_index << 22) | (p << 12)) };
            }
        }

        // follow pde to get pt
        let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};
        let pte = &mut pt[pte_index];

        // if pt entry is not already there
        if !pte.is_flag(0) { // present bit
            // no pt entry yet -> create one

            // set pte
            pte.set_read_write(true); // read/write
            pte.set_privelege_level(false); // kernel only
            pte.set_caching(false); // write-back
            pte.set_address(phys); // point to frame
            pte.set_present(true); // present
        }

        on();
    }

    // map the given paddr for temporary use by the kernel and return a mut reference to the frame
    // NOTE: should only be called on the current address space
    // NOTE: noone should use more than 256 kmappings at a time
    // because it assumes that the PD is at 0x802000
    fn kmap(&mut self, paddr: usize) -> &mut Frame {
        // get next unmapped address
        let next = self.kmap_index;

        // increment
        if next == 255 {
            // unmap all if we have run out
            for i in 0..256 {
                self.unmap(0xC00000 + i * 0x1000);
            }
            self.kmap_index = 0;
        } else {
            self.kmap_index += 1;
        }

        // get associated address
        let vaddr = 0xC00000 + (next as usize) * 0x1000;

        // map the frame
        self.map(paddr, vaddr);

        unsafe { &mut *(vaddr as *mut Frame) }
    }

    // remove any mapping for this virtual address
    // NOTE: should only be called on the current address space
    // because it assumes that the PD is at 0x802000
    fn unmap(&mut self, virt: usize) {
        // TODO: completely clear entries (not just P bit) unless paging out

        off();

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let pd = unsafe {&mut *(0x802000 as *mut VMTable)};

        if pd[pde_index].is_flag(0) { // present bit
            let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};

            // unmap and deallocate frame
            pt[pte_index].free(virt >= 0xD00000);

            // if page table is now empty,
            // unmap and deallocate it
            if (0..1024).all(|i| !pt[i].is_flag(0)) {
                pd[pde_index].free(virt >= 0xC00000);
            }
        }

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        on();
    }

    pub fn activate(&mut self) {
        off();
        unsafe {
            vmm_on(self.page_dir);
            VMM_ON = true;
        }
        on();
    }

    // remove all non-kernel mappings
    // NOTE: must run while this address space is active
    pub fn clear(&mut self) {
        let pd = unsafe {&mut *(0x802000 as *mut VMTable)};

        // for each present PDE, remove all
        // mappings associated with it
        for pde_index in 3..1024 {
            if pd[pde_index].is_flag(0) { // present bit
                for pte_index in 0..1024 {
                    self.unmap((pde_index << 22) | (pte_index << 12));
                }
            }
        }
    }
}

impl Drop for AddressSpace {
    // NOTE: cannot run while the address space is active
    fn drop(&mut self) {
        // free the page directory
        Frame::free(self.page_dir >> 12);
    }
}

impl PagingEntry {
    // get a blank entry
    const fn new() -> PagingEntry {
        PagingEntry {
            entry: 0,
        }
    }

    // common flags

    fn set_present(&mut self, value: bool) {
        self.set_flag(0, value);
    }

    // true = read/write, false = read-only
    fn set_read_write(&mut self, value: bool) {
        self.set_flag(1, value);
    }

    // true = all, false = kernel mode only
    fn set_privelege_level(&mut self, value: bool) {
        self.set_flag(2, value);
    }

    // true = write-through, false = write-back
    fn set_caching(&mut self, value: bool) {
        self.set_flag(3, value);
    }

    // general ops

    fn set_flag(&mut self, index: u8, value: bool) {
        let mask = 1_usize << index;
        let entry = self.entry & !mask;
        self.entry = entry | if value {mask} else {0};
    }

    fn set_address(&mut self, address: usize) {
        let addr = address & 0xFFFF_F000;
        let entry = self.entry & 0x0000_0FFF;

        self.entry = addr | entry;
    }

    // queries about the entry

    fn is_flag(&self, index: u8) -> bool {
        ((self.entry >> index) & 1) == 1
    }

    fn get_address(&self) -> usize {
        self.entry & 0xFFFF_F000
    }

    // free frame pointed to if dealloc and mark this entry not present
    // if it is not already so
    fn free(&mut self, dealloc: bool) {
        off();
        if self.is_flag(0) {
            if dealloc {
                Frame::free(self.get_address() >> 12);
            }

            // mark this entry not present
            self.set_present(false);
        }
        on();
    }
}

impl VMTable {
    // returns a reference to a new VMTable and its paddr
    fn new() -> (&'static mut VMTable, usize) {
        // allocate a frame
        let paddr = Frame::alloc();

        let mut table: &mut VMTable = unsafe {
            // if VM is on, kmap this frame
            // otherwise use the paddr
            let frame = if !CURRENT_PROCESS.is_null() && VMM_ON {
                (*CURRENT_PROCESS).addr_space.kmap(paddr)
            } else {
                &mut *(paddr as *mut Frame)
            };

            transmute(frame)
        };

        // clear the frame
        for i in 0..1024 {
            table[i] = PagingEntry::new();
        }

        (table, paddr)
    }
}

impl Index<usize> for VMTable {
    type Output = PagingEntry;

    fn index<'a>(&'a self, index: usize) -> &'a PagingEntry {
        unsafe { transmute::<&usize, &PagingEntry>(
                &transmute::<&VMTable, &Frame>(self)[index]) }
    }
}

impl IndexMut<usize> for VMTable {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut PagingEntry {
        unsafe { transmute::<&mut usize, &mut PagingEntry>(
                &mut transmute::<&mut VMTable, &mut Frame>(self)[index]) }
    }
}

pub fn init() {
    // create the first two PDEs
    unsafe {
        // PDE0
        PDE0.set_present(true); // present
        PDE0.set_read_write(true); // read/write
        PDE0.set_privelege_level(false); // kernel only
        PDE0.set_caching(false); // write-back
        PDE0.set_address(Frame::alloc()); // alloc a new PT

        // map first 4MiB except page 0
        let pt0 = &mut *(PDE0.get_address() as *mut VMTable);
        for i in 1..1024 {
            pt0[i] = PagingEntry::new();
            pt0[i].set_present(true); // present
            pt0[i].set_read_write(true); // read/write
            pt0[i].set_privelege_level(false); // kernel only
            pt0[i].set_caching(false); // write-back
            pt0[i].set_address(i << 12); // PD paddr
        }

        // PDE1
        PDE1.set_present(true); // present
        PDE1.set_read_write(true); // read/write
        PDE1.set_privelege_level(false); // kernel only
        PDE1.set_caching(false); // write-back
        PDE1.set_address(Frame::alloc()); // alloc a new PT

        // map second 4MiB
        let pt1 = &mut *(PDE1.get_address() as *mut VMTable);
        for i in 0..1024 {
            pt1[i] = PagingEntry::new();
            pt1[i].set_present(true); // present
            pt1[i].set_read_write(true); // read/write
            pt1[i].set_privelege_level(false); // kernel only
            pt1[i].set_caching(false); // write-back
            pt1[i].set_address((i+1024) << 12); // PD paddr
        }
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    bootlog!("virt mem inited\n");
}

// handle for vmm for asm
#[no_mangle]
pub unsafe extern "C" fn vmm_page_fault(/*context: *mut KContext,*/ fault_addr: usize) {
    // segfault! should be very rare with rust
    // first 13MiB are reserved by kernel
    if fault_addr < 0xD00000 {
        // TODO: only kill that process
        panic!("{:?} [segmentation violation @ 0x{:X}]",
               *CURRENT_PROCESS, fault_addr);
    }

    //printf!("page fault {:X}\n", fault_addr);

    if !CURRENT_PROCESS.is_null() && VMM_ON {
        (*CURRENT_PROCESS).addr_space.map(Frame::alloc(), fault_addr);
    } else {
        panic!("Page fault with no current process");
    }

    // memclr an alloced frame?
    let page = &mut *(fault_addr as *mut Frame);
    for i in 0..1024 {
        page[i] = 0;
    }

    //printf!("page fault done {:X}\n", fault_addr);
}
