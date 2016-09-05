//! A simple module for creating an OFS disk image

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::cmp::min;
use std::mem;

/// The size of a sector in bytes
pub const SECTOR_SIZE: u32 = 512; // bytes

const INODE_SIZE: usize = 128; // bytes
const DNODE_SIZE: usize = 512; // bytes

/// Represents an OFS disk image, which can be written to a file
pub struct OFSImage {
    /// The number of inodes in the fs
    num_inodes: u32,

    /// The number of dnodes in the fs
    num_dnodes: u32,

    /// A list of inodes
    inodes: Vec<Option<Inode>>,

    /// A list of dnodes
    dnodes: Vec<Option<Dnode>>,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Inode {
    name: [u8; 12], // file name (up to 12B)
    uid: u32, // owner UID
    gid: u32, // group GID
    user_perm: u8, // user permissions
    group_perm: u8, // group permissions
    all_perm: u8, // everyone permissions
    flags: u8, // various flags
    size: u32, // file size in bytes NOTE: for now this must a multiple of 4
    data: u32, // index of Dnode with contents
    created: u32, // date created
    modified: u32, // date last modified
    links: [u32; 22],
}

impl Inode {
    fn new() -> Inode {
        Inode {
            name: [0; 12],
            uid: 0,
            gid: 0,
            user_perm: 0,
            group_perm: 0,
            all_perm: 0,
            flags: 0,
            size: 0,
            data: 0,
            created: 0,
            modified: 0,
            links: [0; 22],
        }
    }
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct Dnode {
    bytes: [u8; DNODE_SIZE], // contents
}

impl Clone for Dnode {
    fn clone(&self) -> Dnode {
        let mut cloned = [0; DNODE_SIZE];
        for d in 0..cloned.len() {
            cloned[d] = self.bytes[d];
        }
        Dnode { bytes: cloned }
    }
}

impl OFSImage {
    /// Create a new OFSImage with the given numbers of inodes and dnodes.
    pub fn new(num_inodes: u32, num_dnodes: u32) -> OFSImage {
        let mut inodes = Vec::with_capacity(num_inodes as usize);
        let mut dnodes = Vec::with_capacity(num_dnodes as usize);

        for _ in 0..num_inodes {
            inodes.push(None)
        }

        for _ in 0..num_dnodes {
            dnodes.push(None)
        }

        OFSImage {
            num_inodes: num_inodes,
            num_dnodes: num_dnodes,
            inodes: inodes,
            dnodes: dnodes,
        }
    }

    /// Allocate an inode in the file system and return mutable reference to it.
    pub fn alloc_inode(&mut self) -> &mut Inode {
        let first_free = (0..self.num_inodes)
            .find(|i| self.inodes.get_mut(*i as usize).unwrap().is_none())
            .unwrap_or_else(|| panic!("No inodes left!")) as usize;

        *self.inodes.get_mut(first_free).unwrap() = Some(Inode::new());

        self.inodes
            .get_mut(first_free)
            .unwrap()
            .as_mut()
            .unwrap()
    }

    /// Allocate a dnode in the file system and return its dnode index.
    pub fn alloc_dnode(&mut self) -> usize {
        let first_free = (0..self.num_dnodes)
            .find(|i| self.dnodes.get_mut(*i as usize).unwrap().is_none())
            .unwrap_or_else(|| panic!("No dnodes left!")) as usize;

        *self.dnodes.get_mut(first_free).unwrap() = Some(Dnode { bytes: [0; DNODE_SIZE] });

        first_free
    }

    /// Get the `i`th dnode.
    ///
    /// # Panics
    /// if the `i`th dnode is not allocated
    pub fn get_dnode(&mut self, i: u32) -> &mut Dnode {
        self.dnodes
            .get_mut(i as usize)
            .unwrap()
            .as_mut()
            .unwrap_or_else(|| panic!("dnode {} is not allocated", i))
    }

    /// Copy the file (on the host machine) to the OFS image. No guarantees are made about
    /// the order of inodes or dnodes chosen.
    pub fn add_file(&mut self, file: &str) {
        // open the file and read it
        if !Path::new(file).is_file() {
            println!("Error! No such file or directory '{}'.", file);
            exit(1);
        }
        let mut f = File::open(file).ok().unwrap();

        // get the file size (bytes)
        let size = f.metadata().ok().unwrap().len();

        // create the dnodes
        let mut remaining = size;
        let mut first_dnode = 0;

        let mut next_dnode = first_dnode;
        while remaining > DNODE_SIZE as u64 {
            // allocate a dnode
            let prev_dnode = next_dnode;
            next_dnode = self.alloc_dnode();
            if remaining == size {
                // don't forget where the file begins ;)
                first_dnode = next_dnode;
            } else {
                // chain the dnodes
                let dnode = self.get_dnode(prev_dnode as u32);
                dnode.bytes[DNODE_SIZE - 1] = ((next_dnode >> 24) & 0xFF) as u8;
                dnode.bytes[DNODE_SIZE - 2] = ((next_dnode >> 16) & 0xFF) as u8;
                dnode.bytes[DNODE_SIZE - 3] = ((next_dnode >> 8) & 0xFF) as u8;
                dnode.bytes[DNODE_SIZE - 4] = ((next_dnode) & 0xFF) as u8;
            }

            // read DNODE_SIZE - 4 bytes
            let mut buf = [0; DNODE_SIZE - 4]; // last 32-bit word is for the next-ptr
            let _ = f.read_exact(&mut buf).ok().unwrap();

            // copy to the dnode
            let mut dnode_bytes = [0; DNODE_SIZE];
            for i in 0..(DNODE_SIZE - 4) {
                dnode_bytes[i] = buf[i];
            }

            let dnode = self.get_dnode(next_dnode as u32);
            dnode.bytes = dnode_bytes;

            // update counter
            remaining -= (DNODE_SIZE - 4) as u64;
        }

        // create the last dnode
        let last_dnode_idx = self.alloc_dnode() as u32;
        {
            let last_dnode = self.get_dnode(last_dnode_idx);
            let _ = f.read_exact(&mut last_dnode.bytes);
        }

        if size > DNODE_SIZE as u64 {
            let second_last = self.get_dnode(next_dnode as u32);
            second_last.bytes[DNODE_SIZE - 1] = ((last_dnode_idx >> 24) & 0xFF) as u8;
            second_last.bytes[DNODE_SIZE - 2] = ((last_dnode_idx >> 16) & 0xFF) as u8;
            second_last.bytes[DNODE_SIZE - 3] = ((last_dnode_idx >> 8) & 0xFF) as u8;
            second_last.bytes[DNODE_SIZE - 4] = ((last_dnode_idx) & 0xFF) as u8;
        }

        // create the inode
        let inode = self.alloc_inode();

        let basename_str = Path::new(file)
            .canonicalize()
            .expect("Cannot canonicalize file path!")
            .file_name()
            .unwrap()
            .to_os_string();

        let basename = basename_str.to_str()
            .unwrap();

        inode.size = size as u32;
        for i in 0..min(12, basename.len()) {
            inode.name[i] = basename.as_bytes()[i];
        }
        inode.data = first_dnode as u32;

        println!("Added file '{}' to image.", basename);
    }

    /// Write the bytes of this OFSImage to the given file.
    pub fn burn(&self, file: &str) {
        let outfile = File::create(file)
            .expect(format!("Cannot write to file '{}'", file).as_str());
        let mut wbuf = BufWriter::new(outfile); // buffer writes

        // metadata sector
        let mut buf = [0; SECTOR_SIZE as usize];

        // magic bytes
        buf[0] = 'O' as u8;
        buf[1] = 'F' as u8;
        buf[2] = 'S' as u8;

        // inode count
        buf[7] = ((self.num_inodes >> 24) & 0xFF) as u8;
        buf[6] = ((self.num_inodes >> 16) & 0xFF) as u8;
        buf[5] = ((self.num_inodes >> 8) & 0xFF) as u8;
        buf[4] = ((self.num_inodes) & 0xFF) as u8;

        // dnode count
        buf[11] = ((self.num_dnodes >> 24) & 0xFF) as u8;
        buf[10] = ((self.num_dnodes >> 16) & 0xFF) as u8;
        buf[9] = ((self.num_dnodes >> 8) & 0xFF) as u8;
        buf[8] = ((self.num_dnodes) & 0xFF) as u8;

        let _ = wbuf.write_all(&buf);

        // inode bitmap
        let inode_bitmap_sectors =
            (self.num_inodes as f64 / SECTOR_SIZE as f64 / 8.0).ceil() as usize;
        for i in 0..inode_bitmap_sectors {
            buf = [0; SECTOR_SIZE as usize];

            // create this section of the bit map
            for j in 0..(SECTOR_SIZE * 8) as usize {
                if self.inodes.get(i * SECTOR_SIZE as usize * 8 + j).unwrap().is_some() {
                    buf[j / 8] |= (1 << (j % 8)) as u8;
                }
            }

            let _ = wbuf.write_all(&buf);
        }

        // dnode bitmap
        let dnode_bitmap_sectors =
            (self.num_dnodes as f64 / SECTOR_SIZE as f64 / 8.0).ceil() as usize;
        for i in 0..dnode_bitmap_sectors {
            buf = [0; SECTOR_SIZE as usize];

            // create this section of the bit map
            for j in 0..(SECTOR_SIZE * 8) as usize {
                if self.dnodes.get(i * SECTOR_SIZE as usize * 8 + j).unwrap().is_some() {
                    buf[j / 8] |= (1 << (j % 8)) as u8;
                }
            }

            let _ = wbuf.write_all(&buf);
        }

        // inodes
        let inodes_per_sector = SECTOR_SIZE as usize / INODE_SIZE;
        let mut inode_ct = 0;
        let mut inode_buf = [Inode::new(); SECTOR_SIZE as usize / INODE_SIZE];

        for inode_opt in self.inodes.iter() {
            inode_buf[inode_ct % inodes_per_sector] = match *inode_opt {
                Some(ref inode) => (*inode).clone(),
                None => Inode::new(),
            };

            inode_ct += 1;

            if inode_ct % (SECTOR_SIZE as usize / INODE_SIZE) == 0 {
                let _ = wbuf.write_all(unsafe {
                    &mem::transmute::<_, [u8; SECTOR_SIZE as usize]>(inode_buf)
                });
            }
        }

        // dnodes
        let dnodes_per_sector = SECTOR_SIZE as usize / DNODE_SIZE;
        let mut dnode_ct = 0;
        let mut dnode_buf = [Dnode { bytes: [0; DNODE_SIZE] }; SECTOR_SIZE as usize / DNODE_SIZE];

        for dnode_opt in self.dnodes.iter() {
            dnode_buf[dnode_ct % dnodes_per_sector] = match *dnode_opt {
                Some(ref dnode) => (*dnode).clone(),
                None => Dnode { bytes: [0; DNODE_SIZE] },
            };

            dnode_ct += 1;

            if dnode_ct % dnodes_per_sector == 0 {
                let _ = wbuf.write_all(unsafe {
                    &mem::transmute::<_, [u8; SECTOR_SIZE as usize]>(dnode_buf)
                });
            }
        }

        // flush the buffer
        let _ = wbuf.flush();

        println!("Burned image to file '{}'.", file);
    }
}
