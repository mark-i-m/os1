// Will eventually be a user process, but there is a long way to go
use super::Process;
use super::ready_queue;

use core::option::Option::{Some, None};

use super::super::vga::window::{Window, Color};

use super::super::data_structures::concurrency::{StaticSemaphore};

static mut s1: StaticSemaphore = StaticSemaphore::new(3);

pub fn run(this: &Process) -> usize {
    // clear the screen
    Window::clear_screen();

    let mut w0 = Window::new(40, 10, (4, 10));

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    w0.set_cursor((1, 1));
    w0.set_bg_color(Color::LightGreen);
    w0.set_fg_color(Color::Red);
    w0.put_str("Hello World!");

    w0.set_cursor((2,1));
    w0.put_str("from ");
    w0.put_str(this.name);

    ready_queue::make_ready(Process::new("p1", self::run2));

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

    let mut i = 0;
    while i < 6000 {
        ready_queue::make_ready(Process::new("loop_proc", self::run3));
        i += 1;
    }

    0
}

pub fn run3(this: &Process) -> usize {
    let mut w0 = Window::new(3, 1, (0, 0));

    w0.set_bg_color(Color::Yellow);
    w0.paint();

    w0.set_cursor((0,0));
    w0.set_fg_color(Color::Red);

    match this.pid % 4 {
        0 => w0.put_str("---"),
        1 => w0.put_str("\\\\\\"),
        2 => w0.put_str("|||"),
        3 => w0.put_str("///"),
        _ => panic!("The impossible has happened!"),
    }

    0
}
