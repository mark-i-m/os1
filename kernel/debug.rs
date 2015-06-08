// Allow the user to print to qemu serial console
//
// Barrowed from krzysz00/rust-kernel/kernel/console.rs

use core::fmt::{Write,Error};
use core::result::Result;
use core::str::StrExt;

use machine::{inb, outb};

use super::interrupts::{on, off};

const PORT: u16 = 0x3F8;
pub struct Debug;

impl Debug {
    pub fn write_bytes(&self, bytes: &[u8]) {
        // disable interrupts
        off();

        for b in bytes {
            unsafe {
                while inb(PORT + 5) & 0x20 == 0 { }
                outb(PORT, *b);
            }
        }

        // enable interrupts
        on();
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
        use ::core::fmt::Write;
        let _ = write!($crate::debug::Debug, $($arg)*);
    })
}
