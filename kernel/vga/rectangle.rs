//! A module for primitive rectangle drawing

pub use super::vga::Color;

use core::fmt::{Write,Error};

use super::vga::VGA;

/// Ann abstraction for drawing text inside a bounded box
///
/// A rectangle has boundaries. Drawing outside those boundaries will
/// not render any text on the screen.
///
/// Furthermore, a rectangle can draw a string, rather than just a single
/// character.
#[derive(Copy, Clone)]
pub struct Rectangle {
    height: usize,
    width: usize,
    pos: (usize, usize),
    cursor: (usize, usize), // position relative to the rectangle
    vga: VGA,
}

impl Rectangle {
    /// Create a new Rectangle.
    pub fn new(w: usize, h: usize, (row, col): (usize, usize)) -> Rectangle {
        Rectangle {
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

    /// Paint the rectangle with the background color
    pub fn paint(&mut self) {
        self.vga.fill_rect(self.pos, self.height, self.width);
    }

    /// Draw the character at the cursor relative to the corner of rectangle
    /// if the character is inside the rectangle.
    pub fn put_char(&mut self, ch: char) {
        let (cr, cc) = self.cursor;
        if cr < self.height && cc < self.width {
            self.vga.put_char(ch);
        }
    }

    /// Render the string at the cursor using word wrapping.
    pub fn put_str(&mut self, s: &str) {
        let mut first = true;
        for word in s.split(" ") {
            // word wrapping
            let len = word.len();
            let (mut cr, mut cc) = self.cursor;

            if cc + len >= self.width && len < self.width {
                cr += 1;
                cc = 0;
                self.set_cursor((cr, cc));
            }

            // draw a space before the word
            if cc > 0 && !first {
                self.put_char(' ');

                // advance
                if cc + 1 >= self.width {
                    cr += 1;
                    cc = 0;
                } else {
                    cc += 1;
                }
                self.set_cursor((cr,cc));
            }

            for ch in word.chars() {
                // take char of special characters
                match ch {
                    '\n' => {
                        // don't draw a new line, just move the cursor
                        cr += 1;
                        cc = 0;
                    },
                    _ => {
                        // draw char
                        self.put_char(ch);

                        // advance
                        if cc + 1 >= self.width {
                            cr += 1;
                            cc = 0;
                        } else {
                            cc += 1;
                        }
                    }
                }

                // move cursor
                self.set_cursor((cr, cc));

                first = false;
            }
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
    pub fn set_cursor(&mut self, (crow, ccol): (usize, usize)) {
        let (row,col) = self.pos;

        self.cursor = (crow, ccol);
        self.vga.set_cursor((row+crow, col+ccol));
    }
}

impl Write for Rectangle {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.put_str(s);
        Result::Ok(())
    }
}
