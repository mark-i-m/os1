// Module for detecting available physical memory

use core::iter::Iterator;

use alloc::boxed::Box;

use super::physmem::Frame;

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

struct Region {
    start: usize,
    end: usize,
    usable: bool,
    next: Option<Box<Region>>,
}

pub struct RegionMap {
    list: Box<Region>,
    index: usize,
}

impl Region {
    // Recursively create regions and return a chain
    // that stretches from start to the end of the 32-bit addr space
    // each iteration is O(n), but it doesn't matter much because n < 2*20
    pub fn new(start: usize) -> Region {
        if let Some((length,region_type)) = Region::find(start) {
            // avoid overflow and underflow
            let end = if start == 0 {
                (start + length) - 1
            } else {
                (start - 1) + length
            };
            //bootlog!("--{:x},{:x}==\n",start, length);

            Region {
                start: start,
                end: end,
                usable: region_type == 1,
                next: if end < 0xFFFF_FFFF {
                    Some(box Region::new(end + 1))
                } else {
                    None
                },
            }
        } else {
            let end = Region::find_next(start);
            //bootlog!("--{:x},{:x}==\n",start, end - start + 1);
            Region {
                start: start,
                end: end,
                usable: false,
                next: if end < 0xFFFF_FFFF {
                    Some(box Region::new(end + 1))
                } else {
                    None
                },
            }
        }
    }

    // find the length and type of the most restrictive
    // mapping for this addr
    fn find(addr: usize) -> Option<(usize, usize)> {
        let mut most = 0;
        let mut length = 0;
        let mut region_start = 0;
        for i in 0..(memory_map_count as usize) {
            let start = memory_map[i].base_l as usize;
            let end = if memory_map[i].base_l == 0 {
                (memory_map[i].base_l + memory_map[i].length_l) - 1
            } else {
                (memory_map[i].base_l - 1) + memory_map[i].length_l
            } as usize;
            let region_type = memory_map[i].region_type as usize;

            if start <= addr && addr <= end && region_type > most {
                length = end - start;
                most = region_type;
                region_start = start;
                if most > 1 { break; }
            }
        }

        if most == 0 { // not found
            None
        } else {
            Some((length+1-(addr-region_start), most))
        }
    }

    fn find_next(addr: usize) -> usize {
        let mut least = 0xFFFF_FFFF;
        for i in 0..(memory_map_count as usize) {
            let start = memory_map[i].base_l as usize;

            if addr < start && start < least {
                least = start;
            }
        }

        least - 1
    }

    // recursively print mappings
    // used for debugging
    #[allow(dead_code)]
    pub fn dump(&self) {
        bootlog!("0x{:08X} - 0x{:08X} : {}\n",
                 self.start,
                 self.end,
                 if self.usable { "usable" } else { "reserved" }
                );
        if let Some(ref next) = self.next {
            next.dump();
        }
    }
}

impl RegionMap {
    // produce a map of all memory after start
    // this is a 32-bit OS, so we only have to deal with 4GB
    pub fn new(start: *mut Frame) -> RegionMap {
        // starting at start, check if there is an available
        // memory region and add a mapping for it.
        RegionMap {
            list: box Region::new(start as usize),
            index: 0,
        }
    }
}

impl Iterator for RegionMap {
    type Item = (usize, usize);

    // Returns the index of the next avail frame and number of available frames OR None if there aren't any more
    fn next(&mut self) -> Option<(usize, usize)> {
        // find the current region
        let mut region = &self.list;
        for _ in 0..self.index {
            region = if let Some(ref r) = region.next {
                r
            } else {
                // end of list
                return None;
            }
        }

        // find the first useable region
        while !region.usable {
            if let Some(ref r) = region.next {
                region = r;
                self.index += 1;
            } else {
                // No more useable regions
                return None;
            }
        }

        // round up to get first page aligned address in the region
        let f_start = (region.start + 0xFFF) & !0xFFF;

        // go to the end or the next reserved region
        while let Some(ref r) = region.next {
            self.index += 1;

            if r.usable {
                region = r;
            } else {
                break;
            }
        }

        Some((f_start >> 12, (region.end - f_start + 1) >> 12))
    }
}
