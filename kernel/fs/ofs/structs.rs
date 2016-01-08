//! A module for struct definition for OFS

/// A 4B representation of the date for use
/// in the OS
#[repr(C, packed)]
struct OFSDate {
    date: usize,
}

/// A single OFS Inode
#[repr(C, packed)]
struct Inode {
    name: [u8; 12],         // file name (up to 12B)
    uid: usize,             // owner UID
    gid: usize,             // group GID
    size: usize,            // file size in bytes
    data: usize,            // index of Dnode with contents
    created: OFSDate,       // date created
    modified: OFSDate,      // date last modified
    parents: [usize; 3],    // indices of nodes pointing to this node
                            //     upto 3, but the last can be a link
                            //     to more pointers
    children: [usize; 4],   // indices of nodes pointed to by this node
                            //     upto 4, but the last can be a link
                            //     to more pointers
}

