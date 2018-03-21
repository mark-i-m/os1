//! A module for VGA buffer manipulation

use core::ops::{Index, IndexMut};

use machine::outb;

/// Colors for VGA display
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Pink = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightPink = 13,
    Yellow = 14,
    White = 15,
}

/// Number of rows in the VGA buffer
pub const ROWS: usize = 25;
/// Number of columns in the VGA buffer
pub const COLS: usize = 80;

/// The VGA buffer
pub static mut VGA_BUFFER: *mut VGABuff = (0xb8000 as *mut VGABuff);

/// The VGA command port
const VGA_CMD: u16 = 0x3d4;

/// The VGA data port
const VGA_DATA: u16 = 0x3d5;

/// Represents a single character in the VGA buffer.
/// The first byte represents the ASCII character. The next
/// 4 bits represent the background color. The last 4 bits
/// represent the forground color.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VGAChar {
    ch: u8,
    color: u8,
}

/// Abstracts the VGA buffer
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VGABuff {
    buff: [[VGAChar; COLS]; ROWS],
}

/// Safe wrapper around unsafe VGA buffer.
/// Provides the abstraction of a cursor and "screen"
#[derive(Copy, Clone)]
pub struct VGA {
    fg: Color,
    bg: Color,
    cursor: (usize, usize),
}

impl VGAChar {
    /// Set the forground color of the character
    pub fn set_fg(&mut self, c: Color) {
        let bg = self.color & 0xF0;
        self.color = bg | (c as u8);
    }

    /// Set the background color of the character
    pub fn set_bg(&mut self, c: Color) {
        let fg = self.color & 0x0F;
        self.color = ((c as u8) << 4) | fg;
    }

    /// Set the ASCII character
    pub fn set_char(&mut self, ch: char) {
        self.ch = ch as u32 as u8;
    }
}

/// Make the VGA buffer indexable
impl Index<(usize, usize)> for VGABuff {
    type Output = VGAChar;

    fn index<'a>(&'a self, (row, col): (usize, usize)) -> &'a VGAChar {
        &self.buff[row][col]
    }
}

/// Make the VGA buffer indexable
impl IndexMut<(usize, usize)> for VGABuff {
    fn index_mut<'a>(&'a mut self, (row, col): (usize, usize)) -> &'a mut VGAChar {
        &mut self.buff[row][col]
    }
}

impl VGA {
    /// Create a new VGA handle
    pub fn new() -> VGA {
        VGA {
            fg: Color::White,
            bg: Color::Black,
            cursor: (0, 0),
        }
    }

    /// Clear the rectangle and paint it with the background color
    pub fn fill_rect(&self, (row, col): (usize, usize), height: usize, width: usize) {
        // check bounds
        if row >= ROWS || col >= COLS {
            return;
        }

        let rend = if row + height > ROWS {
            ROWS
        } else {
            row + height
        };
        let cend = if col + width > COLS {
            COLS
        } else {
            col + width
        };

        for r in row..rend {
            for c in col..cend {
                unsafe {
                    (*VGA_BUFFER)[(r, c)].set_bg(self.bg);
                    (*VGA_BUFFER)[(r, c)].set_fg(self.fg);
                    (*VGA_BUFFER)[(r, c)].set_char(' ');
                }
            }
        }
    }

    /// Paint the whole screen black
    pub fn clear_screen(&self) {
        self.fill_rect((0 as usize, 0 as usize), ROWS as usize, COLS as usize);
    }

    /// Set character at the cursor
    pub fn put_char(&self, ch: char) {
        unsafe {
            (*VGA_BUFFER)[self.cursor].set_bg(self.bg);
            (*VGA_BUFFER)[self.cursor].set_fg(self.fg);
            (*VGA_BUFFER)[self.cursor].set_char(ch);
        }
    }

    /// Set foreground color of the cursor
    pub fn set_fg(&mut self, color: Color) {
        self.fg = color;
    }

    /// Set background color of the cursor
    pub fn set_bg(&mut self, color: Color) {
        self.bg = color;
    }

    /// Set position of the cursor
    pub fn set_cursor(&mut self, (row, col): (usize, usize)) {
        // update struct
        self.cursor = (row, col);

        // draw new cursor if it is on the screen
        let cursor_offset = (row * COLS + col) as u16;
        let lsb = (cursor_offset & 0xFF) as u8;
        let msb = (cursor_offset >> 8) as u8;

        if row < ROWS && col < COLS {
            unsafe {
                outb(VGA_CMD, 0x0f);
                outb(VGA_DATA, lsb);
                outb(VGA_CMD, 0x0e);
                outb(VGA_DATA, msb);
            }
        }
    }
}
