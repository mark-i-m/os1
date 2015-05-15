// Allow the user to print to qemu serial console
//
// Barrowed from krzysz00/rust-kernel/kernel/console.rs
// TODO: lock before writing; I haven't written a lock yet

use machine;
use core::fmt::{Write,Error};
use core::result::Result;
use core::str::StrExt;

const PORT: u16 = 0x3F8;
pub struct Debug;

impl Debug {
    pub fn write_bytes(&self, bytes: &[u8]) {
        for b in bytes {
            unsafe {
                while machine::inb(PORT + 5) & 0x20 == 0 {};
                machine::outb(PORT, *b);
            }
        }
    }
}

impl Write for Debug {
    #[inline]
    fn write_str(&mut self, data: &str) -> Result<(), Error> {
        self.write_bytes(data.as_bytes());
        Result::Ok(())
    }
}

pub fn puts(string: &str) {
    let _ = Debug.write_str(string);
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        use ::core::fmt::Write;
        let _ = write!($crate::debug::Debug, $($arg)*);
    })
}
