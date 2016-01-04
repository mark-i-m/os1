//! A module for showing user input boxes

use string::String;
use vec::Vec;

pub trait InputElement {
    fn get_str(&mut self) -> String;
}

/// A single-line input box for keyboard input
pub struct TextBox {
    chars: Vec<char>,
}

/// A multi-line input box for keyboard input with
/// word-wrap
pub struct TextArea {
    chars: Vec<char>,
}

// TODO
//impl InputElement for TextBox {
//
//}
//
//impl InputElement for TextArea {
//
//}
