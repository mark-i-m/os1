//! A module for os1 FS (OFS)

pub mod file;

mod internals;
mod hw;

use internals::*;

/// A safe handle on the file system for all needed operations.
pub struct OFSHandle<B: BlockDevice> {
    fs: Arc<Semaphore<OFS<B>>>,
}

impl<B: BlockDevice> OFSHandle<B> {
    /// Create a new handle on the fs using the device
    ///
    /// NOTE: we do not have to lock the device now, because ownership transfers to the handle :D
    pub fn new(mut device: B) -> OFSHandle<B> {
        let mut buf = BlockDataBuffer::new(SECTOR_SIZE);

        device.read_fully(0, &mut buf);

        // get the metadata sector
        let meta = unsafe { buf.get_ref::<Metadata>(0).clone() };

        // check the magic
        if &*meta.get_magic_str() != "OFS\x00" {
            panic!("This is not an OFS volume!");
        }

        OFSHandle {
            fs: Arc::new(Semaphore::new(OFS {
                                            device: device,
                                            meta: meta,
                                        },
                                        1)),
        }
    }

    /// Open the file with the given inode number and return a handle to it.
    pub fn open_read(&mut self, inode: usize) -> Result<File<B>, Error> {
        // TODO mark as read
        // lock the fs
        let mut fs = self.fs.down();

        if !fs.is_file(inode) {
            Err(Error::new("No such file or directory"))
        } else {
            let i = fs.get_inode(inode);
            let d = i.data;
            // printf!("Open file {:?}: i {}, d {}\n", i.name, inode, d);
            Ok(File {
                inode_num: inode,
                inode: i,
                offset: 0,
                offset_dnode: d,
                ofs: self.fs.clone(),
            })
        }
    }

    /// Open the file with the given inode number and return a handle to it.
    pub fn open_write(&mut self, inode: usize) -> Result<File<B>, Error> {
        // TODO mark as write
        // lock the fs
        let mut fs = self.fs.down();

        if !fs.is_file(inode) {
            Err(Error::new("No such file or directory"))
        } else {
            let i = fs.get_inode(inode);
            let d = i.data;
            // printf!("Open file {:?}: i {}, d {}\n", i.name, inode, d);
            Ok(File {
                inode_num: inode,
                inode: i,
                offset: 0,
                offset_dnode: d,
                ofs: self.fs.clone(),
            })
        }
    }

    /// Create a link (directed edge) from file `a` to file `b`. `a` and `b` are the inode number
    /// of the files.
    pub fn link(&mut self, a: usize, b: usize) -> Option<Error> {
        // TODO
        // TODO: make sure that they are not already linked
        Some(Error::new("OFS is read-only until I think about consistency..."))
    }

    /// Remove a link (directed edge) from file `a` to file `b`. `a` and `b` are the inode number
    /// of the files.
    pub fn unlink(&mut self, a: usize, b: usize) -> Option<Error> {
        // TODO
        // TODO: make sure that they are already linked
        Some(Error::new("OFS is read-only until I think about consistency..."))
    }

    /// Return metadata for the file with inode `a` or None if the file does not exist
    pub fn stat(&self, a: usize) -> Option<Inode> {
        let mut fs = self.fs.down();
        if fs.is_file(a) {
            Some(fs.get_inode(a))
        } else {
            None
        }
    }

    /// Create a new file and a link from `a` to it. `a` is the inode number of the file. Return
    /// the inode number of the new file.
    pub fn new_file(&mut self) -> Result<usize, Error> {
        return Err(Error::new("OFS is read-only until I think about consistency..."));

        let mut fs = self.fs.down();

        let inode_num = fs.get_new_inode();
        let dnode_num = fs.get_new_dnode();
        let inode = Inode {
            name: UNNAMED,
            uid: 0,
            gid: 0,
            user_perm: 7,
            group_perm: 0,
            all_perm: 0,
            flags: 0,
            size: 0,
            data: dnode_num,
            created: OFSDate::now(),
            modified: OFSDate::now(),
            links: [0; 22],
        };

        let mut tmp = BlockDataBuffer::new(mem::size_of::<Inode>());
        unsafe {
            *tmp.get_ref_mut(0) = inode;
        }

        let inode_start_offset = fs.get_inode_offset(inode_num);
        fs.device.write_fully(inode_start_offset, &mut tmp);

        // TODO: link the file to the the current working file of the creating process

        Ok(inode_num)
    }

    /// Delete file `a`. `a` is the inode number of the file.
    pub fn delete_file(&mut self, a: usize) -> Option<Error> {
        // TODO
        // Remove links
        // Remove Inode
        // Remove Dnodes
        // TODO: what if a file is deleted while another process is writing to it?
        Some(Error::new("OFS is read-only until I think about consistency..."))
    }

    pub fn get_inode_number(&mut self, path: &str) -> Option<usize> {
        // TODO: get inode number from absolute file path
        // TODO: Path object?
        None
    }
}
