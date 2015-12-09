// Process address spaces

use core::ops::{Index, IndexMut};

use core::intrinsics::transmute;

use super::physmem::Frame;

use super::super::machine::{invlpg, vmm_on};

use super::super::interrupts::{on, off};

use super::super::process::CURRENT_PROCESS;

// TODO: memclr an alloced frame

// The address space of a single process
pub struct AddressSpace {
    page_dir: &'static mut VMTable,
}

// A single entry in a page directory or table
// 31                 12  9       0
// [                    000 flags  ]
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
        let pd = VMTable::new();

        // clear the table
        let mut i = 0;
        while i < 1024 {
            pd[i] = PagingEntry::new();
            i += 1;
        }

        let mut addr_space = AddressSpace { page_dir: pd };

        // Direct map the beginning of memory
        //
        // TODO

        // Omit page 0, so we can detect segfaults.
        // segfaults should be very rare with rust,
        // but better safe than sorry...
        //i = 1<<12;
        //while i < unsafe { super::physmem::PHYS_MEM_END as usize } {
        //    addr_space.map(i, i);
        //    i += 1<<12;
        //}

        addr_space
    }

    fn map(&mut self, phys: usize, virt: usize) {
        off();

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        // if pd entry is not already there
        let pde = &mut self.page_dir[pde_index];
        if !pde.is_flag(0) { // present bit
            // no pd entry yet -> create one

            // get a new page table
            let new_pt = VMTable::new();

            // set pde
            pde.set_present(true); // present
            pde.set_read_write(true); // read/write
            pde.set_privelege_level(false); // kernel only
            pde.set_caching(false); // write-back
            pde.set_address(new_pt as *mut VMTable as usize); // point to pt
        }

        // follow pde to get pt
        let pt = unsafe { &mut *(pde.get_address() as *mut VMTable) };
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

    fn unmap(&mut self, virt: usize) {
        off();

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let pde = &mut self.page_dir[pde_index];
        let pt = unsafe { &mut *(pde.get_address() as *mut VMTable) };

        {
            let pte = &mut pt[pte_index];
            let frame = unsafe { &mut *(pte.get_address() as *mut Frame) };

            // unmap and deallocate frame
            pte.set_present(false);
            // TODO
            //if virt >= unsafe { super::physmem::PHYS_MEM_END as usize } {
            //    frame.free();
            //}
        }

        // if page table is now empty,
        // unmap and deallocate it
        let mut i = 0;
        let mut empty = true;
        while i < 1024 {
            if pt[i].is_flag(0) {
                empty = false;
            }
            i += 1;
        }

        if empty {
            pde.set_present(false);
            let pt_frame = unsafe { &mut *(pde.get_address() as *mut Frame) };
            //TODO
            //pt_frame.free();
        }

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        on();
    }

    pub fn activate(&self) {
        off();
        unsafe { vmm_on(self.page_dir as *const VMTable as usize) }
        on();
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        // for each pde
        let mut pde_index = 0;
        while pde_index < 1024 {
            let pde = &mut self.page_dir[pde_index];
            // if the pde is present
            if pde.is_flag(0) { // present bit
                // get the page table
                let pt = unsafe { &mut *(pde.get_address() as *mut VMTable) };

                // attempt to free each pte
                let mut pte_index = 0;
                while pte_index < 1024 {
                    let pte = &mut pt[pte_index];
                    let virt = pde_index << 22 | pte_index << 12;

                    // TODO

                    //pte.free(virt >= unsafe { super::physmem::PHYS_MEM_END as usize });
                    pte_index += 1;
                }

                // free the page table
                //pde.free(true);
                //TODO
            }

            pde_index += 1;
        }

        off();

        // free the page directory
        // TODO
        //unsafe{ transmute::<&mut VMTable, &mut Frame>(self.page_dir).free(); }

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
                // free the frame
                let frame = unsafe { &mut *(self.get_address() as *mut Frame) };
                //frame.free();
                //TODO
            }

            // mark this entry not present
            self.set_present(false);
        }
        on();
    }
}

impl VMTable {
    fn new() -> &'static mut VMTable {
        unsafe { transmute(Frame::alloc()) }
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

// handle for vmm for asm
#[no_mangle]
pub unsafe extern "C" fn vmm_page_fault(/*context: *mut KContext,*/ fault_addr: usize) {
    // segfault! should be very rare with rust
    if fault_addr < 0x1000 {
        // TODO: only kill that process
        unsafe {
            panic!("{:?} [segmentation violation @ 0x{:X}]",
                   *CURRENT_PROCESS, fault_addr);
        }
    }

    (*CURRENT_PROCESS).addr_space.map(Frame::alloc() as *mut Frame as usize, fault_addr);
}
