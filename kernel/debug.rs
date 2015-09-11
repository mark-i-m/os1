// Allow the user to print to qemu serial console
//
// Barrowed from krzysz00/rust-kernel/kernel/console.rs

use core::fmt::{Write,Error};
use core::result::Result;
use core::str::StrExt;

use machine::{inb, outb};


const PORT: u16 = 0x3F8;
pub struct Debug;

impl Debug {
    pub fn write_bytes(&self, bytes: &[u8]) {
        for b in bytes {
            unsafe {
                while inb(PORT + 5) & 0x20 == 0 { }
                outb(PORT, *b);
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

#[allow(dead_code)]
// Print to console
pub fn puts(string: &str) {
    let _ = Debug.write_str(string);
}

// Print to console
#[macro_export]
macro_rules! printf {
    ($($arg:tt)*) => ({
        // disable interrupts
        use ::core::fmt::Write;
        use ::interrupts::{on, off};
        off();
        let _ = write!($crate::debug::Debug, $($arg)*);
        // enable interrupts
        on();
    })
}

// Version of printf for use BEFORE processes are inited
#[macro_export]
macro_rules! bootlog {
    ($($arg:tt)*) => ({
        use ::core::fmt::Write;
        let _ = write!($crate::debug::Debug, $($arg)*);
    })
}
