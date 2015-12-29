//! A module process address spaces

use super::super::super::concurrency::StaticSemaphore;
use super::super::super::interrupts::{on, off};
use super::super::super::machine::{invlpg, vmm_on};
use super::super::super::process::{CURRENT_PROCESS, State};
use super::super::super::process::proc_table::PROCESS_TABLE;
use super::super::super::static_linked_list::StaticLinkedList;
use super::super::physmem::Frame;
use super::{SHARED_PDES, NUM_SHARED, PD_ADDRESS, KMAP_ADDRESS, USER_ADDRESS, VMM_ON};
use super::structs::{VMTable, PagingEntry};

/// The address space of a single process
pub struct AddressSpace {
    /// paddr of this addr space's PD
    page_dir: usize,

    /// the index of the first un-kmapped page
    kmap_index: u8,

    /// list of page-share requests
    ///
    /// Format: `(PID, paddr)`
    share_req: StaticLinkedList<(usize, usize)>,

    /// a lock for the address space
    ///
    /// NOTE: to prevent deadlocks, *ALWAYS* acquire
    /// this lock before locks on frames.
    lock: StaticSemaphore,
}

impl AddressSpace {
    /// Create a new address space and set up the PD.
    pub fn new() -> AddressSpace {
        // allocate a frame
        let (pd, pd_paddr) = VMTable::new();

        // copy the first entries of the pd to direct map
        unsafe {
            let mut i = 0;
            for pde in &SHARED_PDES {
                pd[i] = pde.clone();
                i += 1;
            }
        }

        // point the third PDE at the PD to map all page tables
        // so PD is at vaddr PD_ADDRESS
        unsafe {
            pd[NUM_SHARED].set_present(true); // present
            pd[NUM_SHARED].set_read_write(true); // read/write
            pd[NUM_SHARED].set_privelege_level(false); // kernel only
            pd[NUM_SHARED].set_caching(true); // write-through
            pd[NUM_SHARED].set_address(pd_paddr); // PD paddr
        }

        let a = AddressSpace {
            page_dir: pd_paddr,
            kmap_index: 0,
            share_req: StaticLinkedList::new(),
            lock: StaticSemaphore::new(1),
        };

        a
    }

    /// Map `virt` to `phys` in the address space.
    /// NOTE: should only be called on the current address space
    /// because it assumes that the PD is at PD_ADDRESS
    pub fn map(&mut self, phys: usize, virt: usize) {
        //unsafe {
        //    bootlog!("{:?} [map {:x} -> {:x}]\n", *CURRENT_PROCESS, virt, phys);
        //}

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        self.lock.down();

        let mut pd = unsafe {&mut *PD_ADDRESS};
        let pde = &mut pd[pde_index];

        // invalidate TLB entry
        unsafe { invlpg(virt) };

        // if pd entry is not already there
        if !pde.is_flag(0) { // present bit
            // no pd entry yet => create one

            off();

            // set pde
            pde.set_read_write(true); // read/write
            pde.set_privelege_level(false); // kernel only
            pde.set_caching(false); // write-back
            pde.set_address(Frame::alloc()); // alloc a new frame
            pde.set_present(true); // present

            // clear the pt
            let mut pt = unsafe {&mut *(((NUM_SHARED<<22) | (pde_index<<12)) as *mut VMTable)};
            for p in 0..1024 {
                pt[p] = PagingEntry::new();
                unsafe { invlpg((pde_index << 22) | (p << 12)) };
            }

            on();
        }

        // follow pde to get pt
        let mut pt = unsafe {&mut *(((NUM_SHARED<<22) | (pde_index<<12)) as *mut VMTable)};
        let pte = &mut pt[pte_index];

        // if pt entry is not already there
        if !pte.is_flag(0) { // present bit
            // no pt entry yet -> create one

            off();

            // set pte
            pte.set_read_write(true); // read/write
            pte.set_privelege_level(false); // kernel only
            pte.set_caching(false); // write-back
            pte.set_address(phys); // point to frame
            pte.set_present(true); // present

            on();
        }

        self.lock.up();
    }

    /// Map the given `paddr` for temporary use by the kernel and return a mut reference to the
    /// frame.
    /// NOTE: should only be called on the current address space because it assumes that the PD is
    /// at PD_ADDRESS
    pub fn kmap(&mut self, paddr: usize) -> &mut Frame {
        // get next unmapped address
        let next = self.kmap_index;

        // increment
        if next == 255 {
            // unmap all if we have run out
            for i in 0..256 {
                self.unmap(unsafe { KMAP_ADDRESS } + i * 0x1000);
            }
            self.kmap_index = 0;
        } else {
            self.kmap_index += 1;
        }

        // get associated address
        let vaddr = unsafe { KMAP_ADDRESS } + (next as usize) * 0x1000;

        // map the frame
        self.map(paddr, vaddr);

        unsafe { &mut *(vaddr as *mut Frame) }
    }

    /// Remove any mapping for this virtual address
    /// NOTE: should only be called on the current address space because it assumes that the PD is
    /// at PD_ADDRESS
    pub fn unmap(&mut self, virt: usize) {
        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let pd = unsafe {&mut *PD_ADDRESS};

        self.lock.down();

        if pd[pde_index].is_flag(0) { // present bit
            off();

            let mut pt = unsafe {&mut *(((NUM_SHARED<<22) | (pde_index<<12)) as *mut VMTable)};

            // unmap and deallocate frame
            pt[pte_index].free(virt >= unsafe { USER_ADDRESS });

            // invalidate TLB entry
            unsafe { invlpg(virt) };

            on();

            // if page table is now empty,
            // unmap and deallocate it
            if (0..1024).all(|i| !pt[i].is_flag(0)) {
                pd[pde_index].free(virt >= unsafe { KMAP_ADDRESS });
            }
        }

        self.lock.up();
    }

    /// Returns the current physical address mapped to the given virtual address
    /// or None if it is not mapped.
    ///
    /// NOTE: this has to run while the addresss space is active.
    pub fn v_to_p(&mut self, virt: usize) -> Option<usize> {
        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let pd = unsafe {&mut *PD_ADDRESS};

        self.lock.down();

        let ret = if pd[pde_index].is_flag(0) { // present bit
            let mut pt = unsafe {&mut *(((NUM_SHARED<<22) | (pde_index<<12)) as *mut VMTable)};
            let pte = &pt[pte_index];

            if pte.is_flag(0) {
                Some(pt[pte_index].get_address())
            } else {
                None
            }
        } else {
            None
        };

        self.lock.up();

        ret
    }

    /// Activate the current address space and turn on VM if needed
    pub fn activate(&mut self) {
        off();
        unsafe {
            vmm_on(self.page_dir);
            VMM_ON = true;
        }
        on();
    }

    /// Send a page-share request to the process with PID `pid` for
    /// the frame mapped to `vaddr` in this address space.
    ///
    /// Returns true if the request succeeded, and false if there was
    /// an error (e.g. the process has already died).
    ///
    /// NOTE: this has to run while the addresss space is active.
    pub fn request_share(&mut self, pid: usize, vaddr: usize) -> bool {
        // NOTE: need to lock all memory data structures here...
        self.lock.down();

        // get the process and check that it is alive
        let ret = unsafe { if let Some(p) = PROCESS_TABLE.get(pid) {
            match (*p).get_state() {
                State::TERMINATED => false,
                _ => {
                    // get the paddr of this vaddr
                    if let Some(paddr) = self.v_to_p(vaddr) {
                        // add to its addr_space::share_req list
                        (*p).addr_space.share_req.push_back(((*CURRENT_PROCESS).get_pid(), paddr));
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }};

        self.lock.up();

        ret
    }

    /// Creates a mapping in this process's address space for the
    /// first page-share request from the process with the given `PID`
    /// to the given page. Returns true if success; false otherwise.
    pub fn accept_share(&mut self, pid: usize, vaddr: usize) -> bool {
        // find and remove the right request
        let mut i = 0;
        for req in &self.share_req {
            let (req_pid, _) = *req;
            if req_pid == pid {
                break;
            }
            i += 1;
        }

        // none found
        if i == self.share_req.len() {
            return false;
        }

        // lock the list here
        off();
        let tail = &mut self.share_req.split_off(i);
        let paddr = if let Some((_, pa)) = self.share_req.pop_back() {
            pa
        } else {
            panic!("What?");
        };
        self.share_req.append(tail);
        on();

        // make a mapping
        self.map(paddr, vaddr);

        true
    }

    /// Remove all non-kernel mappings in this address space.
    /// NOTE: must run while this address space is active
    pub fn clear(&mut self) {
        let pd = unsafe {&mut *PD_ADDRESS};

        // for each present PDE, remove all
        // mappings associated with it
        for pde_index in (unsafe { NUM_SHARED } + 1)..1024 {
            if pd[pde_index].is_flag(0) { // present bit
                for pte_index in 0..1024 {
                    self.unmap((pde_index << 22) | (pte_index << 12));
                }
            }
        }
    }
}

impl Drop for AddressSpace {
    /// Deallocate the page directory.
    /// NOTE: cannot run while the address space is active
    fn drop(&mut self) {
        // free the page directory
        Frame::free(self.page_dir >> 12);
    }
}

/// The Rust-side code of the page fault handler.
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

    if !CURRENT_PROCESS.is_null() {
        (*CURRENT_PROCESS).addr_space.map(Frame::alloc(), fault_addr);
    } else {
        panic!("Page fault @ 0x{:X} with no current process", fault_addr);
    }

    // memclr an alloced frame
    let page = &mut *(fault_addr as *mut Frame);
    for i in 0..1024 {
        page[i] = 0;
    }

    //printf!("page fault done {:X}\n", fault_addr);
}
