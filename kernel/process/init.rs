#![allow(dead_code)]

use super::{Process, State, STACK_SIZE, StackPtr, NEXT_ID};
use core::marker::Sync;
use alloc::boxed;
use alloc::boxed::{Box, into_raw};

// The init process
pub struct Init {
    name: &'static str,
    id: usize,
    state: State,
    stack: StackPtr,
    kesp: StackPtr,
}

impl Process for Init {
    fn new (name: &'static str) -> Init {
        let stack = StackPtr::get_stack();
        stack.smash();
        Init {
            id: NEXT_ID.get_then_add(),
            name: name,
            state: State::READY,
            stack: stack,
            kesp: stack.get_kesp(),
        }
    }

    fn run (&self) -> usize {
        use super::super::window::{Window, Color};

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
        w0.put_str(self.name);

        0
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}
