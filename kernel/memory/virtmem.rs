// Process address spaces

use core::ops::{Index, IndexMut, Deref, DerefMut};

use core::intrinsics::transmute;

use super::physmem::Frame;

use super::super::machine::{invlpg, vmm_on};

use super::super::interrupts::{on, off};

use super::super::process::CURRENT_PROCESS;

// shared PDEs direct mapping the first 8MiB
static mut PDE0: PagingEntry = PagingEntry {entry: 0};
static mut PDE1: PagingEntry = PagingEntry {entry: 0};

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
        pd[2].set_caching(false); // write-back
        pd[2].set_address(pd_paddr); // PD paddr

        let a = AddressSpace {
            page_dir: pd_paddr,
            kmap_index: 0,
        };

        // if VM on
        // clean up after kmap
        unsafe {
            if !CURRENT_PROCESS.is_null() {
                (*CURRENT_PROCESS).addr_space.unmap(pd as *mut VMTable as usize);
            }
        }

        a
    }

    // map the va to the pa in the address space
    // NOTE: should only be called on the current address space
    // because it assumes that the PD is at 0x802000
    fn map(&mut self, phys: usize, virt: usize) {
        off();

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let mut pd = unsafe {&mut *(self.page_dir as *mut VMTable)};

        // if pd entry is not already there
        let pde = &mut pd[pde_index];
        if !pde.is_flag(0) { // present bit
            // no pd entry yet => create one

            // set pde
            pde.set_present(true); // present
            pde.set_read_write(true); // read/write
            pde.set_privelege_level(false); // kernel only
            pde.set_caching(false); // write-back
            pde.set_address(Frame::alloc()); // alloc a new frame
        }

        // follow pde to get pt
        let mut pt = *pde;
        let pte = &mut pt[pte_index];

        // if pt entry is not already there
        if !pte.is_flag(0) { // present bit
            // no pt entry yet -> create one

            // set pte
            pte.set_present(true); // present
            pte.set_read_write(true); // read/write
            pte.set_privelege_level(false); // kernel only
            pte.set_caching(false); // write-back
            pte.set_address(phys); // point to frame
        }

        on();
    }

    // map the given paddr for temporary use by the kernel and return a mut reference to the frame
    // kmap can only be called 256 times between calls to activate()
    // NOTE: the mappings are undefined after activate() is called, so it is the caller's
    // responsibility to unmap if needed to free resources
    // NOTE: should only be called on the current address space
    // because it assumes that the PD is at 0x802000
    // unless if VM is off
    fn kmap(&mut self, paddr: usize) -> &mut Frame {
        // get next unmapped address
        let next = self.kmap_index;
        self.kmap_index += 1;

        // get associated address
        let vaddr = 0xC000_0000 + (next as usize) * 0x1000;

        // map the frame
        self.map(paddr, vaddr);

        unsafe { &mut *(vaddr as *mut Frame) }
    }

    // remove any mapping for this virtual address
    // NOTE: should only be called on the current address space
    // because it assumes that the PD is at 0x802000
    fn unmap(&mut self, virt: usize) {
        // TODO: completely clear entries unless paging out
        off();

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let mut pd = unsafe {&mut *(self.page_dir as *mut VMTable)};

        let pde = &mut pd[pde_index];
        let mut pt = *pde;

        // unmap and deallocate frame
        {
            let pte = &mut pt[pte_index];
            pte.set_present(false);
            pte.free(virt >= 0xC000_0000);
        }

        // if page table is now empty,
        // unmap and deallocate it
        if (0..1024).all(|i| !pt[i].is_flag(0)) {
            pde.set_present(false);
            pde.free(true);
        }

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        on();
    }

    pub fn activate(&mut self) {
        off();
        unsafe { vmm_on(self.page_dir) }
        on();

        self.kmap_index = 0;
    }
}

impl Drop for AddressSpace {
    // cannot call unmap because this address space is not active
    fn drop(&mut self) {
        let mut pd = unsafe {&mut *(self.page_dir as *mut VMTable)};

        // for each pde
        for pde_index in 0..1024 {
            let pde = &mut pd[pde_index];
            // if the pde is present
            if pde.is_flag(0) { // present bit
                // get the page table
                let mut pt = *pde;

                // attempt to free each pte
                for pte_index in 0..1024 {
                    let pte = &mut pt[pte_index];
                    let virt = pde_index << 22 | pte_index << 12;
                    pte.free(virt >= 0xC000_0000);
                }

                // free the page table
                pde.free(true);
            }
        }

        off();

        // free the page directory
        Frame::free(self.page_dir >> 12);

        on();
    }
}

impl PagingEntry {
    // get a blank entry
    fn new() -> PagingEntry {
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

impl Deref for PagingEntry {
    type Target = VMTable;

    fn deref<'a>(&'a self) -> &'a VMTable {
        unsafe { & *(self.get_address() as *const VMTable) }
    }
}

impl DerefMut for PagingEntry {
    fn deref_mut<'a>(&'a mut self) -> &'a mut VMTable {
        unsafe { &mut *(self.get_address() as *mut VMTable) }
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
            let frame = if !CURRENT_PROCESS.is_null() {
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
    // TODO: init PDE0 and PDE1
}

// handle for vmm for asm
#[no_mangle]
pub unsafe extern "C" fn vmm_page_fault(/*context: *mut KContext,*/ fault_addr: usize) {
    // segfault! should be very rare with rust
    if fault_addr < 0x1000 {
        // TODO: only kill that process
        panic!("{:?} [segmentation violation @ 0x{:X}]",
               *CURRENT_PROCESS, fault_addr);
    }

    (*CURRENT_PROCESS).addr_space.map(Frame::alloc() as *mut Frame as usize, fault_addr);

    // TODO: memclr an alloced frame?

}
