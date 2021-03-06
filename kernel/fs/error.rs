//! A simple Error object for FS errors

pub struct Error<'err> {
    msg: &'err str,
}

impl<'err> Error<'err> {
    pub fn new(msg: &'err str) -> Error {
        Error { msg: msg }
    }
}
