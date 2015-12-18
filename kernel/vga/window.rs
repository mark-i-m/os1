//! A module for primitive window drawing

use super::vga::VGA;

pub use super::vga::Color;

/// The abstration of a window.
///
/// A window has boundaries. Drawing outside those boundaries will
/// not render any text on the screen.
///
/// Furthermore, a window can draw a string, rather than just a single
/// character.
#[derive(Copy, Clone)]
pub struct Window {
    height: usize,
    width: usize,
    pos: (usize, usize),
    cursor: (usize, usize), // position relative to the window
    vga: VGA,
}

impl Window {
    /// Create a new Window.
    pub fn new(w: usize, h: usize, (row, col): (usize, usize)) -> Window {
        Window {
            height: h,
            width: w,
            pos: (row, col),
            cursor: (row, col),
            vga: VGA::new(),
        }
    }

    /// A static method to clear the screen.
    pub fn clear_screen() {
        let mut buff = VGA::new();
        buff.set_fg(Color::White);
        buff.set_bg(Color::Black);
        buff.clear_screen();
    }

    /// Paint the window with the background color
    pub fn paint(&mut self) {
        self.vga.fill_rect(self.pos, self.height, self.width);
    }

    /// Draw the character at the cursor relative to the corner of window
    /// if the character is inside the window.
    pub fn put_char(&mut self, ch: char) {
        let (cr, cc) = self.cursor;
        if cr < self.height && cc < self.width {
            self.vga.put_char(ch);
        }
    }

    /// Render the string at the cursor
    pub fn put_str(&mut self, s: &str) {
        use core::str::StrExt;

        for ch in s.bytes() {
            // draw char
            self.put_char(ch as char);

            // move cursor
            let (cr, cc) = self.cursor;
            let next_cursor = if (ch as char) == ('\n' as char) ||
                cc + 1 >= self.width {
                (cr + 1, 0)
            } else {
                (cr, cc + 1)
            };

            self.set_cursor(next_cursor);
        }
    }

    /// Set the foreground color
    pub fn set_fg(&mut self, color: Color) {
        self.vga.set_fg(color);
    }

    /// Set the background color
    pub fn set_bg(&mut self, color: Color) {
        self.vga.set_bg(color);
    }

    /// Set the cursor position
    pub fn set_cursor(& mut self, (crow, ccol): (usize, usize)) {
        let (row,col) = self.pos;

        self.cursor = (crow, ccol);
        self.vga.set_cursor((row+crow, col+ccol));
    }
}
