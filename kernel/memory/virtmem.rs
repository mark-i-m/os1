//! A module for virtual memory management
//!
//! The virtual address space of a process consists of all addresses for 0xD0_0000-0xFFFF_FFFF.
//! When a page fault occurs (in kernel or user mode), if the faulting address is in this range,
//! the fault is considered an error, and the process is killed. This is because the first 8MiB are
//! shared, direct-mapped memory and should never produce a page fault. The next 4MiB are mapped to
//! the processes page directory, so if there is a fault in this range, it is fatal, since it means
//! the paging structures have been corrupt. The 13MiB is claimed by the kernel for use by the kmap
//! function, which temporarily maps a page meant to reside in another process's address space.
//! This is needed for creating a new process. Luckly, with Rust, segfaults should be extremely
//! rare.
//!
//! When a page fault occurs for a legal address, the fault handler checks the following cases:
//!
//! 1. Is the page paged out to disk? If so, swap it back in
//! 2. Is the page marked present but read-only? If so, this page is COW, so clone it and mark the
//!    new page read/write
//! 3. Else, allocate a new frame a map the page to it

use core::ops::{Index, IndexMut};

use core::intrinsics::transmute;

use super::physmem::Frame;

use super::super::machine::{invlpg, vmm_on, page_fault_handler};

use super::super::interrupts::{add_trap_handler, on, off};

use super::super::process::CURRENT_PROCESS;

use alloc::boxed::Box;

/// A list of shared PDEs direct mapping the beginning of memory
static mut SHARED_PDES: *mut SharedPDEList = 0 as *mut SharedPDEList;

/// Is VM on?
static mut VMM_ON: bool = false;

/// The address space of a single process
pub struct AddressSpace {
    // paddr of this addr space's PD
    page_dir: usize,

    // the index of the first un-kmapped page
    kmap_index: u8,
}

/// A single entry in a page directory or table
/// ```
/// 31                 12  9       0
/// [                    000 flags  ]
/// ```
#[derive(Copy, Clone)]
#[repr(C,packed)]
struct PagingEntry {
    entry: usize,
}

/// An abstraction of page directories and page tables
#[repr(C,packed)]
struct VMTable {
    entries: [PagingEntry; 1024],
}

/// A data structure to keep track of shared PDEs
struct SharedPDEList {
    pde: PagingEntry,
    next: Option<Box<SharedPDEList>>,
}

impl AddressSpace {
    /// Create a new address space. Each address space has the first
    /// 8MiB direct mapped. The next 4MiB is mapped to the page directory
    pub fn new() -> AddressSpace {
        // allocate a frame
        let (pd, pd_paddr) = VMTable::new();

        // copy the first entries of the pd to direct map
        unsafe {
            let shared_pdes = &*SHARED_PDES;
            for i in 0..shared_pdes.length() {
                pd[i] = shared_pdes[i].get_pde();
            }
        }

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

    /// Map `virt` to `phys` in the address space.
    /// NOTE: should only be called on the current address space
    /// because it assumes that the PD is at 0x802000
    fn map(&mut self, phys: usize, virt: usize) {
        //unsafe {
        //    bootlog!("{:?} [map {:x} -> {:x}]\n", *CURRENT_PROCESS, virt, phys);
        //}

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let mut pd = unsafe {&mut *(0x802000 as *mut VMTable)};
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
            let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};
            for p in 0..1024 {
                pt[p] = PagingEntry::new();
                unsafe { invlpg((pde_index << 22) | (p << 12)) };
            }

            on();
        }

        // follow pde to get pt
        let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};
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
    }

    /// Map the given `paddr` for temporary use by the kernel and return a mut reference to the
    /// frame.
    /// NOTE: should only be called on the current address space because it assumes that the PD is
    /// at 0x802000
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

    /// Remove any mapping for this virtual address
    /// NOTE: should only be called on the current address space because it assumes that the PD is
    /// at 0x802000
    fn unmap(&mut self, virt: usize) {
        // TODO: completely clear entries (not just P bit) unless paging out

        let pde_index = virt >> 22;
        let pte_index = (virt & 0x003F_F000) >> 12;

        let pd = unsafe {&mut *(0x802000 as *mut VMTable)};

        if pd[pde_index].is_flag(0) { // present bit
            off();

            let mut pt = unsafe {&mut *(((2<<22) | (pde_index<<12)) as *mut VMTable)};

            // unmap and deallocate frame
            pt[pte_index].free(virt >= 0xD00000);

            // invalidate TLB entry
            unsafe { invlpg(virt) };

            on();

            // if page table is now empty,
            // unmap and deallocate it
            if (0..1024).all(|i| !pt[i].is_flag(0)) {
                pd[pde_index].free(virt >= 0xC00000);
            }
        }
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

    /// Share the given page with the process with PID `pid`
    pub fn share_page(pid: usize, vaddr: usize) {
        // TODO
    }

    /// Remove all non-kernel mappings in this address space.
    /// NOTE: must run while this address space is active
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
    /// Deallocate the page directory.
    /// NOTE: cannot run while the address space is active
    fn drop(&mut self) {
        // free the page directory
        Frame::free(self.page_dir >> 12);
    }
}

impl PagingEntry {
    /// Get a blank entry
    #[inline(always)]
    const fn new() -> PagingEntry {
        PagingEntry {
            entry: 0,
        }
    }

    // common flags

    /// Set the present bit.
    /// true = present; false = not present
    #[inline(always)]
    fn set_present(&mut self, value: bool) {
        self.set_flag(0, value);
    }

    /// Set the read/write bit.
    /// true = read/write, false = read-only
    #[inline(always)]
    fn set_read_write(&mut self, value: bool) {
        self.set_flag(1, value);
    }

    /// Set the privelege level bit.
    /// true = all, false = kernel mode only
    #[inline(always)]
    fn set_privelege_level(&mut self, value: bool) {
        self.set_flag(2, value);
    }

    /// Set caching bit.
    /// true = write-through, false = write-back
    #[inline(always)]
    fn set_caching(&mut self, value: bool) {
        self.set_flag(3, value);
    }

    // general ops

    /// Set the `index`-th flag to `value`.
    /// true = 1; false = 0
    #[inline(always)]
    fn set_flag(&mut self, index: u8, value: bool) {
        let mask = 1_usize << index;
        let entry = self.entry & !mask;
        self.entry = entry | if value {mask} else {0};
    }

    /// Set the upper 20 bits of the entry to the upper 20 bits of `address`
    #[inline(always)]
    fn set_address(&mut self, address: usize) {
        let addr = address & 0xFFFF_F000;
        let entry = self.entry & 0x0000_0FFF;

        self.entry = addr | entry;
    }

    // queries about the entry

    /// Return true if the `index`-th bit is set
    #[inline(always)]
    fn is_flag(&self, index: u8) -> bool {
        ((self.entry >> index) & 1) == 1
    }

    /// Return the upper 20-bits of the entry
    #[inline(always)]
    fn get_address(&self) -> usize {
        self.entry & 0xFFFF_F000
    }

    /// Free the frame pointed to if `dealloc` and mark this entry not present.
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
    /// Returns a reference to a new `VMTable` and its physical address.
    /// The new table is kmapped if necessary, and is always cleared.
    fn new() -> (&'static mut VMTable, usize) {
        // allocate a frame
        let paddr = Frame::alloc();

        let mut table: &mut VMTable = unsafe {
            // if VM is on, kmap this frame
            // otherwise use the paddr
            let frame = if VMM_ON {
                if !CURRENT_PROCESS.is_null() {
                    (*CURRENT_PROCESS).addr_space.kmap(paddr)
                } else {
                    panic!("VMM_ON with no CURRENT_PROCESS!");
                }
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

/// Make `VMTable` indexable
impl Index<usize> for VMTable {
    type Output = PagingEntry;

    fn index<'a>(&'a self, index: usize) -> &'a PagingEntry {
        unsafe { transmute::<&usize, &PagingEntry>(
                &transmute::<&VMTable, &Frame>(self)[index]) }
    }
}

/// Make `VMTable` indexable
impl IndexMut<usize> for VMTable {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut PagingEntry {
        unsafe { transmute::<&mut usize, &mut PagingEntry>(
                &mut transmute::<&mut VMTable, &mut Frame>(self)[index]) }
    }
}

impl SharedPDEList {
    /// Create `n` PDEs to direct map the memory start from the `i`th page
    pub fn new(n: usize, i: usize) -> *mut SharedPDEList {
        printf!("n: {}, i: {}\n", n, i);
        unsafe {
            // create the page table and pde
            let mut pde = PagingEntry::new();

            pde.set_present(true); // present
            pde.set_read_write(true); // read/write
            pde.set_privelege_level(false); // kernel only
            pde.set_caching(false); // write-back
            pde.set_address(Frame::alloc()); // alloc a new PT

            // map direct map except page 0
            let pt = &mut *(pde.get_address() as *mut VMTable);
            for e in (if i == 0 {1} else {0})..1024 {
                pt[e] = PagingEntry::new();
                pt[e].set_present(true); // present
                pt[e].set_read_write(true); // read/write
                pt[e].set_privelege_level(false); // kernel only
                pt[e].set_caching(false); // write-back
                pt[e].set_address(((i<<10)+e) << 12); // PD paddr
            }

            Box::into_raw(box SharedPDEList {
                pde: pde,
                next: if n == 1 {
                    None
                } else {
                    Some(Box::from_raw(SharedPDEList::new(n-1, i+1)))
                }
            })
        }
    }

    pub fn length(&self) -> usize {
        1 + if let Some(ref next) = self.next {
            next.length()
        } else {
            0
        }
    }

    pub fn get_pde(&self) -> PagingEntry {
        self.pde
    }
}

impl Index<usize> for SharedPDEList {
    type Output = SharedPDEList;

    fn index<'a>(&'a self, index: usize) -> &'a SharedPDEList {
        if index == 0 {
            self
        } else {
            if let Some(ref next) = self.next {
                &next[index - 1]
            } else {
                panic!("Index out of bounds!");
            }
        }
    }
}

impl IndexMut<usize> for SharedPDEList {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut SharedPDEList {
        if index == 0 {
            self
        } else {
            if let Some(ref mut next) = self.next {
                &mut next[index - 1]
            } else {
                panic!("Index out of bounds!");
            }
        }
    }
}

/// Initialize virtual memory management but do not turn on VM.
///
/// This creates the shared page tables that map the first beginning of memory.
/// It also registers the `page_fault_handler`.
///
/// All memory before `start` is direct mapped. The first 4MiB after `start are
/// mapped to the page directory. The next MiB is reserved kernel memory.
/// `start` must be 4MiB aligned.
pub fn init(start: usize) {

    if start % (4<<20) != 0 {
        panic!("virt mem start must be 4MiB aligned");
    }

    // Create the shared PDEs
    // Each PDE maps 4MiB (2^22)
    unsafe {
        SHARED_PDES = SharedPDEList::new(start >> 22, 0);
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    bootlog!("virt mem inited - start addr: 0x{:X}\n", start);
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
