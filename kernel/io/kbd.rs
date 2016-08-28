//! A module for getting keyboard input

use super::super::machine::inb;
use super::super::process::proc_table::PROCESS_TABLE;
use super::super::process::focus::get_focused;
use super::stream::OutputStream;

/// The difference between a capital and lowercase
const CAP: u8 = ('a' as u8) - ('A' as u8);

/// Is this character capital?
static mut SHIFT: bool = false;

/// The keyboard interrupt handler
///
/// Get a character from the keyboard and
/// place it in the buffer of the focused process
/// if that process is alive and has a buffer.
pub fn handler() {
    let focused_pid = get_focused();
    unsafe {
        if let Some(p) = PROCESS_TABLE.get(focused_pid) {
            if let Some(ref mut buff) = (*p).buffer {
                if let Some(key) = get_key() {
                    buff.put(key);
                }
            }
        }
    }
}

/// Determine if this character is capital or not
fn ul(c: char) -> char {
    unsafe { if SHIFT { (c as u8 - CAP) as char } else { c } }
}

/// Get a character from the keyboard
fn get_key() -> Option<char> {
    while unsafe { inb(0x64) } & 1 == 0 {
    }
    let b: u8 = unsafe { inb(0x60) };
    match b {
        0x02...0x0a => Some(('0' as u8 + b - 1) as char),
        0x0b => Some('0'),
        0x0e => Some(8 as char),
        0x10 => Some(ul('q')),
        0x11 => Some(ul('w')),
        0x12 => Some(ul('e')),
        0x13 => Some(ul('r')),
        0x14 => Some(ul('t')),
        0x15 => Some(ul('y')),
        0x16 => Some(ul('u')),
        0x17 => Some(ul('i')),
        0x18 => Some(ul('o')),
        0x19 => Some(ul('p')),
        0x1c => Some('\n'),
        0x1e => Some(ul('a')),
        0x1f => Some(ul('s')),
        0x20 => Some(ul('d')),
        0x21 => Some(ul('f')),
        0x22 => Some(ul('g')),
        0x23 => Some(ul('h')),
        0x24 => Some(ul('j')),
        0x25 => Some(ul('k')),
        0x26 => Some(ul('l')),
        0x2c => Some(ul('z')),
        0x2d => Some(ul('x')),
        0x2e => Some(ul('c')),
        0x2f => Some(ul('v')),
        0x30 => Some(ul('b')),
        0x31 => Some(ul('n')),
        0x32 => Some(ul('m')),
        0x39 => Some(' '),

        // TODO: map other ascii characters
        0x2a | 0x36 => {
            unsafe {
                SHIFT = true;
            }
            None
        }
        0xaa | 0xb6 => {
            unsafe {
                SHIFT = false;
            }
            None
        }

        _ => None,
    }
}
