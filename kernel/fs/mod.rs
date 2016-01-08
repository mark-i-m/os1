//! A module for file system stuff

use io::block::{BlockDevice, BlockDataBuffer};
use io::ide::{IDE, IDEBuf};

pub mod ofs;
pub mod traits;

/// Initialize the file system
pub fn init(mut device: IDE) {
    use string::String;

    let mut buf = IDEBuf::new(4);

    device.read_fully(49, &mut buf);

    unsafe {
        let data = buf.get_ref::<[u8; 4]>(0);
        let mut string = String::new();

        for c in data.iter() {
            string.push(*c as char);
        }

        printf!("string: {}\n", string);
        printf!("data: {:?}\n", &*string);
    }

    printf!("filesystem inited\n");
}
