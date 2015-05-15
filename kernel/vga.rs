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

pub fn clear_screen(background: Color) {
    for i in 0..(80 * 25) {
        unsafe {
            *((0xb8000 + i * 2) as *mut u16) = (background as u16) << 12;
        }
    }
}
