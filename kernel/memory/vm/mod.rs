// Module for virtual memory management

pub use self::process::vmm_page_fault;

pub use self::process::AddressSpace;

mod physmem;
mod process;

pub fn init(mut start: usize, mut end: usize) {
    self::physmem::init(start, end);
}
