//! This module contains some basic functionality that libstd would normally
//! otherwise provide. Most importantly, it defines `rust_begin_unwind` which is
//! used by `panic!`.

use core::fmt;

use debug::Debug;
use machine::cli;

// For bare-bones rust
#[lang = "eh_personality"] extern fn eh_personality() {}

/// This function is used by `panic!` to display an error message.
#[lang = "panic_fmt"]
pub extern fn rust_begin_unwind(args: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
    use core::fmt::Write;
    unsafe { cli(); } // we should no be interrupting any more
    bootlog!("\nPanic at {}:{}: ", file, line);
    let _ = Debug.write_fmt(args);
    bootlog!("\n");
    loop {};
}
