// The idle process

use super::{Process};

// Just waste a quantum and hopefully there will be something to do
#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    loop{}
}
