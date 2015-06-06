use machine;

// colors for VGA display
#[derive(Copy, Clone)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Pink       = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    LightPink  = 13,
    Yellow     = 14,
    White      = 15,
}

// size of VGA buffer
pub const ROWS: usize = 25;
pub const COLS: usize = 80;

// location of vga buffer
const VGA_BUFFER: *mut VGAChar = (0xb8000 as *mut VGAChar);

// represents a single character on the screen
#[derive(Copy,Clone)]
struct VGAChar {
    ch: u8,
    color: u8,
}

// unsafe construct that give direct access to VGA buffer
#[allow(raw_pointer_derive)]
#[derive(Copy, Clone)]
struct VGABuff {
    buff: *mut VGAChar,
}

impl VGABuff {
    fn get_buff() -> VGABuff {
        VGABuff {buff: VGA_BUFFER}
    }

    fn pack_colors(fg: Color, bg: Color) -> u8 {
        ((bg as u8) << 4) | ((fg as u8))
    }

    fn put_char(&self, (r,c): (usize, usize), ch: VGAChar) {
        unsafe {
            *self.buff.offset((r * COLS + c) as isize) = ch;
        }
    }
}

// safe wrapper around unsafe VGA buffer
// Provides the abstraction of a cursor and "screen"
#[derive(Copy, Clone)]
pub struct VGA {
    buff: VGABuff,
    fg: Color,
    bg: Color,
    cursor: (usize, usize),
}

impl VGA {
    // get a new VGA handle
    pub fn get_vga() -> VGA {
        VGA {buff: VGABuff::get_buff(), fg: Color::White, bg: Color::Black, cursor: (0,0)}
    }

    // clear the rect and paint it with the background color
    pub fn clear_screen(&self) {
        self.fill_rect((0 as usize,0 as usize), ROWS as usize, COLS as usize);
    }

    // clear the rect and paint it with the background color
    pub fn fill_rect(&self, pos: (usize, usize), height: usize, width: usize) {
        let colors = VGABuff::pack_colors(self.fg, self.bg);
        let c = VGAChar {ch: ' ' as u32 as u8, color: colors};

        let (row, col) = pos;

        // check bounds
        if row >= ROWS || col >= COLS {return;}

        let rend = if row + height > ROWS { ROWS } else {row + height};
        let cend = if col + width > COLS { COLS } else {col + width};

        for i in row..rend {
            for j in col..cend {
                self.buff.put_char((i as usize,j as usize), c);
            }
        }
    }

    // set character at the cursor
    pub fn put_char(&self, ch: char) {
        let colors = VGABuff::pack_colors(self.fg, self.bg);
        let c = VGAChar {ch: ch as u32 as u8, color: colors};

        self.buff.put_char(self.cursor, c);
    }

    // set foreground color of the cursor
    pub fn set_fg_color(& mut self, color: Color) {
        self.fg = color;
    }

    // set background color of the cursor
    pub fn set_bg_color(& mut self, color: Color) {
        self.bg = color;
    }

    // set position of the cursor
    pub fn set_cursor(& mut self, pos: (usize, usize)) {
        // update struct
        self.cursor = pos;

        // draw new cursor if it is on the screen
        const VGA_CMD: u16 = 0x3d4;
        const VGA_DATA: u16 = 0x3d5;

        let (row, col) = pos;
        let cursor_offset = (row * COLS + col) as u16;
        let lsb = (cursor_offset & 0xFF) as u8;
        let msb = (cursor_offset >> 8) as u8;

        if row < ROWS && col < COLS {
            unsafe {
                machine::outb(VGA_CMD, 0x0f);
                machine::outb(VGA_DATA, lsb);
                machine::outb(VGA_CMD, 0x0e);
                machine::outb(VGA_DATA, msb);
            }
        }
    }
}
