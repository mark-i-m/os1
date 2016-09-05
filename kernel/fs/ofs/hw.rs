//! A module of low-level abstractions for disk-level representations of data for OFS.

use core::mem;
// use core::ops::Index;

use string::String;
use io::ide::SECTOR_SIZE;

/// A 4B representation of the date for use in the OS
type OFSDate = u32;

// TODO: extendable list of elements
// /// A data structure representing a list on the hard
// /// disk, in which the last element may be the dnode
// /// number containing the remaining elements
// pub struct ExtendableList<L: Index<usize>> {
//     list: L,
// }

/// The metadata sector of the partition
#[derive(Clone)]
#[repr(C, packed)]
pub struct Metadata {
    magic: [u8; 4],
    // pub root_inode: usize, // NOTE: inode 0 is always root
    pub num_inode: usize,
    pub num_dnode: usize,
}

/// A single OFS Inode (128B)
#[derive(Clone)]
#[repr(C, packed)]
pub struct Inode {
    pub name: [u8; 12], // file name (up to 12B)
    pub uid: usize, // owner UID
    pub gid: usize, // group GID
    pub user_perm: u8, // user permissions
    pub group_perm: u8, // group permissions
    pub all_perm: u8, // everyone permissions
    pub flags: u8, // various flags
    pub size: usize, // file size in bytes TODO: for now this must a multiple of 4
    pub data: usize, // index of Dnode with contents
    pub created: OFSDate, // date created
    pub modified: OFSDate, // date last modified
    pub links: [usize; 22],
}

/// A single OFS Dnode (512B)
// TODO: deref to slice
#[repr(C, packed)]
pub struct Dnode {
    pub data: [usize; 128],
}

pub const UNNAMED: [u8; 12] = ['u' as u8, 'n' as u8, 'n' as u8, 'a' as u8, 'm' as u8, 'e' as u8,
                               'd' as u8, 0, 0, 0, 0, 0];

impl OFSDate {
    pub fn now() -> OFSDate {
        // TODO
        OFSDate { date: 0 }
    }
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

    /// Return the last word of this dnode. If the file
    /// has another dnode, this will be its index.
    pub fn get_next(&self) -> usize {
        self.data[self.data.len() - 1]
    }
}

impl Clone for Dnode {
    fn clone(&self) -> Dnode {
        let mut data = [0; 128];

        for i in 0..128 {
            data[i] = self.data[i];
        }

        Dnode { data: data }
    }
}
