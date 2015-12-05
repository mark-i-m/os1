// Module for detecting available physical memory

// get memory map at boot time and use it construct RegionMaps
// the BIOS calls needed have to run before we switch to protected mode
extern "C" {
    static memory_map_count: u32;
    static memory_map: [MemoryListEntry; 20];
}

#[repr(C, packed)]
struct MemoryListEntry {
    base_l: u32,
    base_h: u32,
    length_l: u32,
    length_h: u32,
    region_type: u32,
    acpi: u32, // extended
}

pub struct RegionMap;

impl RegionMap {
    // produce a map of all memory after start
    //pub fn new(start: *mut Frame) -> RegionMap {
    //    //TODO
    //}

    //// If the index-th frame is available physical mem, return a reference
    //// to the frame, other wise, none
    //pub fn avail<'a>(index: usize) -> Option<&'a mut Frame> {
    //    // TODO
    //}
}

pub fn init() {
    bootlog!("mem regions {}\n", memory_map_count);

    unsafe { 
        let mut i: usize = 0;
        while i < (memory_map_count as usize) {
            let e = &memory_map[i];
            bootlog!("0x{:X} to 0x{:X} | {}\n", e.base_l, e.base_l + e.length_l - 1, e.region_type);
            i += 1;
        }
    }
}
