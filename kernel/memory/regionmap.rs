//! Module for detecting available physical memory
//!
//! Get memory map at boot time and use it construct `RegionMap`s.
//! The BIOS calls needed have to run before we switch to protected mode,

use core::iter::Iterator;

use super::super::vec::Vec;

use super::physmem::Frame;

extern "C" {
    static memory_map_count: u32;
    static memory_map: [MemoryListEntry; 20];
}

/// Represents an entry in the list of memory regions generated by the E820
/// BIOS call.
#[repr(C, packed)]
struct MemoryListEntry {
    base_l: u32,
    base_h: u32,
    length_l: u32,
    length_h: u32,
    region_type: u32,
    acpi: u32, // extended
}

/// Represents a region of memory
struct Region {
    start: usize,
    end: usize,
    usable: bool,
}

/// Represents a map of memory regions
pub struct RegionMap {
    list: Vec<Region>,
    index: usize,
}

/// Find the length and type of the most restrictive
/// mapping for `addr` based on the E820 BIOS call
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

/// Find the beginning address of the next region of memory
/// based on the E820 BIOS call
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

impl Region {
    /// Print this region for debugging purposes
    #[allow(dead_code)]
    pub fn dump(&self) {
        bootlog!("0x{:08X} - 0x{:08X} : {}\n",
                 self.start,
                 self.end,
                 if self.usable { "usable" } else { "reserved" }
                );
    }
}

impl RegionMap {
    /// Produce a map of all memory after start.
    /// This is a 32-bit OS, so we only have to deal with 4GB
    pub fn new(start_frame: *mut Frame) -> RegionMap {
        // starting at start, check if there is an available
        // memory region and add a mapping for it.

        let mut start = start_frame as usize;

        let mut rlist = Vec::new();

        loop {
            let end;

            let region = if let Some((length,region_type)) = find(start) {
                // avoid overflow and underflow
                end = if start == 0 {
                    (start + length) - 1
                } else {
                    (start - 1) + length
                };
                //bootlog!("--{:x},{:x}==\n",start, length);

                Region {
                    start: start,
                    end: end,
                    usable: region_type == 1,
                }
            } else {
                end = find_next(start);
                //bootlog!("--{:x},{:x}==\n",start, end - start + 1);

                Region {
                    start: start,
                    end: end,
                    usable: false,
                }
            };

            rlist.push(region);

            if end < 0xFFFF_FFFF {
                start = end + 1;
            } else {
                break;
            }
        }

        RegionMap {
            list: rlist,
            index: 0,
        }
    }

    /// Print out the memory map for debugging purposes
    #[allow(dead_code)]
    pub fn dump(&self) {
        for i in 0..self.list.len() {
            self.list[i].dump();
        }
    }
}

/// Make `RegionMap` iterable for convenience
impl Iterator for RegionMap {
    type Item = (usize, usize);

    /// Returns the index of the next avail frame and number of
    /// available frames OR None if there aren't any more.
    fn next(&mut self) -> Option<(usize, usize)> {
        // start and end region indices
        let mut start = self.index;
        let mut end;

        // find the next usable region
        while start < self.list.len() && !self.list[start].usable {
            start += 1;
        }

        // end of list
        if start == self.list.len() {
            return None;
        }

        // find as many contiguous usable regions as possible
        end = start + 1;

        while end < self.list.len() && self.list[end].usable {
            end += 1;
        }

        // update index
        self.index = end;

        // generate bounds
        let f_start = self.list[start].start;
        let f_end = self.list[end-1].end;

        Some((f_start >> 12, (f_end - f_start + 1) >> 12))
    }
}
