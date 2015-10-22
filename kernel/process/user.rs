// Will eventually be a user process, but there is a long way to go
use super::Process;
use super::ready_queue;

use core::option::Option::None;

use super::super::vga::window::{Window, Color};

pub fn run(this: &Process) -> usize {
    // clear the screen
    Window::clear_screen();

    let mut w0 = Window::new(40, 10, (4, 10));

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    super::proc_yield(None);

    w0.set_cursor((1, 1));
    w0.set_bg_color(Color::LightGreen);
    w0.set_fg_color(Color::Red);
    w0.put_str("Hello World!");

    w0.set_cursor((2,1));
    w0.put_str("from ");
    w0.put_str(this.name);

    ready_queue::make_ready(Process::new("p1", self::run2));

    loop{}

    0
}

pub fn run2(this: &Process) -> usize {
    let mut w0 = Window::new(20, 10, (10, 40));

    w0.set_bg_color(Color::White);
    w0.paint();

    w0.set_cursor((1, 1));
    w0.set_fg_color(Color::Black);
    w0.put_str("Hello World!");

    w0.set_cursor((2,1));
    w0.put_str("from ");
    w0.put_str(this.name);

    0
}
