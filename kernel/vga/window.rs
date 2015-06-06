use super::vga::{VGA};

pub use super::vga::Color;

// Defines the abstration of a window
#[derive(Copy, Clone)]
pub struct Window {
    height: usize,
    width: usize,
    pos: (usize, usize),
    cursor: (usize, usize), // position relative to the window
    vga: VGA,
}

impl Window {
    pub fn new(w: usize, h: usize, pos: (usize, usize)) -> Window {
        Window {height: h, width: w, pos: pos, cursor: pos, vga: VGA::get_vga()}
    }

    // clear the screen
    pub fn clear_screen() {
        let mut buff = VGA::get_vga();
        buff.set_fg_color(Color::White);
        buff.set_bg_color(Color::Black);
        buff.clear_screen();
    }

    // paint the window
    pub fn paint(& mut self) {
        self.vga.fill_rect(self.pos, self.height, self.width);
    }

    // draw the character at the cursor relative to the corner of window
    pub fn put_char(& mut self, ch: char) {
        // check bounds
        let (cr, cc) = self.cursor;
        if cr < self.height && cc < self.width {
            self.vga.put_char(ch);
        }
    }

    pub fn put_str(& mut self, s: &str) {
        use core::str::StrExt;

        for ch in s.bytes() {
            // draw char
            self.put_char(ch as char);

            // move cursor
            let (cr, cc) = self.cursor;
            let next_cursor = if cc + 1 >= self.width {
                    (cr + 1, 0)
                } else {
                    (cr, cc + 1)
                };

            self.set_cursor(next_cursor);
        }
    }

    pub fn set_fg_color(& mut self, color: Color) {
        self.vga.set_fg_color(color);
    }

    pub fn set_bg_color(& mut self, color: Color) {
        self.vga.set_bg_color(color);
    }

    pub fn set_cursor(& mut self, pos: (usize, usize)) {
        self.cursor = pos;

        let (row,col) = self.pos;
        let (cr, cc) = self.cursor;

        self.vga.set_cursor((row+cr, col+cc));
    }
}
