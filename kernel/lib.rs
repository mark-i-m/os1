#![feature(no_std,lang_items,core)]
#![no_std]

#![crate_type = "staticlib"]
#![crate_name = "rustcode"]

#[macro_use]
extern crate core;
extern crate rlibc;

use core::fmt;

#[no_mangle]
pub fn kernel_main() {

}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
pub extern fn rust_begin_unwind(args: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
//    use core::fmt::Write;
//    log!("\r\nPanic at {}:{}: ", file, line);
//    let _ = console::Console.write_fmt(args);
    loop {};
}
