// Module for detecting available physical memory

struct RegionMap;

impl RegionMap {
    // produce a map of all memory after start
    pub fn new(start: *mut Frame) -> RegionMap {
        //TODO
    }

    // If the index-th frame is available physical mem, return a reference
    // to the frame, other wise, none
    pub fn avail<'a>(index: usize) -> Option<&'a mut Frame> {
        // TODO
    }
}
