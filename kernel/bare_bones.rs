use core::fmt;

// For bare-bones rust
#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

#[allow(unused_variables)]
#[lang = "panic_fmt"]
pub extern fn rust_begin_unwind(args: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
    loop {};
}
