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

// unsafe construct that give direct access to VGA buffer
pub const ROWS: isize = 25;
pub const COLS: isize = 80;

#[derive(Copy,Clone)]
struct VGAChar {
    ch: u8,
    color: u8,
}

struct VGABuff {
    buff: *mut VGAChar,
}

impl VGABuff {
    fn get_buff() -> VGABuff {
        VGABuff {buff: (0xb8000 as *mut VGAChar)}
    }

    fn pack_colors(fg: Color, bg: Color) -> u8 {
        ((bg as u8) << 4) | ((fg as u8))
    }

    fn put_char(&self, (r,c): (isize, isize), ch: VGAChar) {
        unsafe {
            *self.buff.offset(r * COLS + c) = ch;
        }
    }

    fn get_char(&self, (r,c): (isize, isize)) -> &VGAChar {
        unsafe {
            &*self.buff.offset(r * COLS + c)
        }
    }
}

// safe wrapper around unsafe VGA buffer
pub struct VGA {
    buff: VGABuff,
    fg: Color,
    bg: Color,
    cursor: (isize, isize),
}

impl VGA {
    // get a new VGA handle
    pub fn get_vga() -> VGA {
        VGA {buff: VGABuff::get_buff(), fg: Color::White, bg: Color::Black, cursor: (0,0)}
    }

    // clear the screen and paint it with the background color
    pub fn fill(&self) {
        let colors = VGABuff::pack_colors(self.fg, self.bg);
        let c = VGAChar {ch: ' ' as u32 as u8, color: colors};

        for i in 0..ROWS {
            for j in 0..COLS {
                self.buff.put_char((i,j), c);
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
    pub fn set_cursor(& mut self, pos: (isize, isize)) {
        self.cursor = pos;
    }
}


pub fn clear_screen(background: Color) {
    let mut buff = VGA::get_vga();

    buff.set_bg_color(background);
    buff.fill();

    buff.set_cursor((10, 40));
    buff.set_bg_color(Color::LightGreen);
    buff.set_fg_color(Color::Red);
    buff.put_char('a');
}
