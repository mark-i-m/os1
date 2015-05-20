use core::fmt;

use debug::Debug;

// For bare-bones rust
#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
pub extern fn rust_begin_unwind(args: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
    use core::fmt::Write;
    printf!("\nPanic at {}:{}: ", file, line);
    let _ = Debug.write_fmt(args);
    printf!("\n");
    loop {};
}
