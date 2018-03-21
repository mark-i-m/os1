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

pub use self::addr_space::{vmm_page_fault, AddressSpace};

mod addr_space;
mod structs;

use super::super::interrupts::add_trap_handler;
use super::super::machine::page_fault_handler;
use super::super::static_linked_list::StaticLinkedList;
use super::physmem::Frame;
use self::structs::{PagingEntry, VMTable};

/// A list of shared PDEs direct mapping the beginning of memory
static mut SHARED_PDES: StaticLinkedList<PagingEntry> = StaticLinkedList::new();

/// The number of shared PDEs (for convenience)
static mut NUM_SHARED: usize = 0;

/// The virtual address to which the PD is mapped
static mut PD_ADDRESS: *mut VMTable = 0 as *mut VMTable;

/// The beginning of kmap memory
static mut KMAP_ADDRESS: usize = 0;

/// The beginning of user memory
static mut USER_ADDRESS: usize = 0;

/// Is VM on?
static mut VMM_ON: bool = false;

/// Create `n` PDEs to direct map the memory start from the `i`th page.
/// unsafe because it deals with raw pointers and static muts.
unsafe fn init_shared_pdes(n: usize, mut i: usize) {
    for _ in 0..n {
        // create the page table and pde
        let mut pde = PagingEntry::new();

        pde.set_present(true); // present
        pde.set_read_write(true); // read/write
        pde.set_privelege_level(false); // kernel only
        pde.set_caching(false); // write-back
        pde.set_address(Frame::alloc()); // alloc a new PT

        // map direct map except page 0
        let pt = &mut *(pde.get_address() as *mut VMTable);
        for e in (if i == 0 { 1 } else { 0 })..1024 {
            pt[e] = PagingEntry::new();
            pt[e].set_present(true); // present
            pt[e].set_read_write(true); // read/write
            pt[e].set_privelege_level(false); // kernel only
            pt[e].set_caching(false); // write-back
            pt[e].set_address(((i << 10) + e) << 12); // PD paddr
        }

        SHARED_PDES.push_back(pde);

        i += 1;
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
    if start % (4 << 20) != 0 {
        panic!("virt mem start must be 4MiB aligned");
    }

    // Create the shared PDEs
    // Each PDE maps 4MiB (2^22)
    unsafe {
        init_shared_pdes(start >> 22, 0);
        NUM_SHARED = SHARED_PDES.len();
        PD_ADDRESS = ((NUM_SHARED << 22) | (NUM_SHARED << 12)) as *mut VMTable;
        KMAP_ADDRESS = (NUM_SHARED + 1) << 22;
        USER_ADDRESS = ((NUM_SHARED + 1) << 22) + (1 << 20);
    }

    // Register page fault handler
    add_trap_handler(14, page_fault_handler, 0);

    bootlog!("virt mem inited - start addr: 0x{:X}\n", start);
}
