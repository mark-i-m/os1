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

/// A single OFS Inode (128B)
#[derive(Clone)]
#[repr(C, packed)]
pub struct Inode {
    pub name: [u8; 12],         // file name (up to 12B)
    pub uid: usize,             // owner UID
    pub gid: usize,             // group GID
    pub user_perm: u8,          // user permissions
    pub group_perm: u8,         // group permissions
    pub all_perm: u8,           // everyone permissions
    pub flags:u8,               // various flags
    pub size: usize,            // file size in bytes
    pub data: usize,            // index of Dnode with contents
    pub created: OFSDate,       // date created
    pub modified: OFSDate,      // date last modified
    //pub parents: ExtendableList<[usize; 10]>,// An extendable list of parent nodes
    //pub children: ExtendableList<[usize; 12]>,// An extendable list of child nodes
    pub parents: [usize; 10],
    pub children: [usize; 12],
}

/// A single OFS Dnode (128B)
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
