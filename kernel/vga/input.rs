//! A module for showing user input boxes

use alloc::string::String;

use core::fmt::Write;

use process::focus::focus;
use process::CURRENT_PROCESS;
use super::rectangle::Rectangle;

pub trait InputElement {
    fn get_str(&mut self) -> String;
}

/// A multi-line input box for keyboard input with
/// word-wrap
pub struct TextArea {
    string: String,
    tbox: Rectangle,
}

/// A single-line input box for keyboard input
pub struct TextBox {
    tbox: TextArea,
}

impl TextArea {
    /// Get a new empty textbox with the given dimensions and position
    pub fn new(w: usize, h: usize, pos: (usize, usize)) -> TextArea {
        TextArea {
            string: String::new(),
            tbox: Rectangle::new(w, h, pos),
        }
    }
}

impl InputElement for TextArea {
    /// Keep accepting characters until the user hits return
    fn get_str(&mut self) -> String {
        let me = unsafe { &mut *CURRENT_PROCESS };
        me.accept_kbd(3);

        self.tbox.paint();
        self.tbox.set_cursor((0, 0));

        // get focus
        focus(None);

        let buffer = me.buffer.as_mut().expect("Oh no! Why isn't there a buffer?");
        loop {
            match buffer.next() {
                Some('\n') => break,
                Some('\x08') => {
                    // backspace
                    let _ = self.string.pop();
                    self.tbox.paint();
                    self.tbox.set_cursor((0, 0));
                    let _ = write!(&mut self.tbox, "{}", self.string);
                }
                Some(c) => {
                    self.string.push(c);
                    self.tbox.paint();
                    self.tbox.set_cursor((0, 0));
                    let _ = write!(&mut self.tbox, "{}", self.string);
                }
                None => {}
            }
        }

        self.string.clone()
    }
}

impl TextBox {
    /// Get a new empty textbox with the given width and position
    pub fn new(w: usize, pos: (usize, usize)) -> TextBox {
        TextBox { tbox: TextArea::new(w, 1, pos) }
    }
}

impl InputElement for TextBox {
    fn get_str(&mut self) -> String {
        self.tbox.get_str()
    }
}
