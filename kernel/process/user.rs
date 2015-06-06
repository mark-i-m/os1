// Will eventually be a user process, but there is a long way to go
use super::Process;

use core::option::Option::None;

pub fn run(this: &Process) -> usize {
    use super::super::window::{Window, Color};

    // clear the screen
    Window::clear_screen();

    let mut w0 = Window::new(40, 10, (4, 10));

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    unsafe{super::proc_yield(None);}

    w0.set_cursor((1, 1));
    w0.set_bg_color(Color::LightGreen);
    w0.set_fg_color(Color::Red);
    w0.put_str("Hello World!");

    w0.set_cursor((2,1));
    w0.put_str("from ");
    w0.put_str(this.name);

    0
}
