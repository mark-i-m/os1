//! A module of low-level abstractions for disk-level representations of data for OFS.

use core::mem;
use core::ops::Index;

use string::String;
use io::ide::SECTOR_SIZE;

/// A 4B representation of the date for use in the OS
#[derive(Clone)]
#[repr(C, packed)]
pub struct OFSDate {
    date: usize,
}

/// A data structure representing a list on the hard
/// disk, in which the last element may be the dnode
/// number containing the remaining elements
pub struct ExtendableList<L: Index<usize>> {
    list: L,
}

/// The metadata sector of the partition
#[derive(Clone)]
#[repr(C, packed)]
pub struct Metadata {
    magic: [u8; 4],
    pub root_inode: usize,
    pub num_inode: usize,
    pub num_dnode: usize,
}

/// A single OFS Inode
#[derive(Clone)]
#[repr(C, packed)]
pub struct Inode {
    name: [u8; 12],         // file name (up to 12B)
    uid: usize,             // owner UID
    gid: usize,             // group GID
    user_perm: u8,          // user permissions
    group_perm: u8,         // group permissions
    all_perm: u8,           // everyone permissions
    flags:u8,               // various flags
    size: usize,            // file size in bytes
    data: usize,            // index of Dnode with contents
    created: OFSDate,       // date created
    modified: OFSDate,      // date last modified
    //parents: ExtendableList<[usize; 10]>,// An extendable list of parent nodes
    //children: ExtendableList<[usize; 12]>,// An extendable list of child nodes
    parents: [usize; 10],
    children: [usize; 12],
}

/// A single OFS Dnode
#[repr(C, packed)]
pub struct Dnode {
    pub data: [usize; 128],
}

impl OFSDate {
    // TODO
}

impl Metadata {
    /// Get the magic string of as a `String`
    pub fn get_magic_str(&self) -> String {
        let mut magic = String::new();

        magic.push(self.magic[0] as char);
        magic.push(self.magic[1] as char);
        magic.push(self.magic[2] as char);
        magic.push(self.magic[3] as char);

        magic
    }
}

impl Inode {
    /// Get the number of inodes in a sector
    pub fn inodes_per_sector() -> usize {
        SECTOR_SIZE / mem::size_of::<Inode>()
    }

    /// Get the filename as a string
    pub fn get_filename(&self) -> String {
        let mut name = String::new();

        for i in 0..12 {
            name.push(self.name[i] as char);
        }

        name
    }
}

impl Dnode {
    /// Get the number of dnodes in a sector
    pub fn dnodes_per_sector() -> usize {
        SECTOR_SIZE / mem::size_of::<Dnode>()
    }
}

impl Clone for Dnode {
    fn clone(&self) -> Dnode {
        let mut data = [0; 128];

        for i in 0..128 {
            data[i] = self.data[i];
        }

        Dnode {
            data: data,
        }
    }
}
