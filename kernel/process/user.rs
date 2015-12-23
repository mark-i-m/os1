//! A module for user processes
// So far it just contains some test code

use super::Process;
use super::ready_queue;

use super::super::vga::window::{Window, Color};

use super::super::concurrency::{StaticSemaphore};

// useful constants
const ROWS: usize = 25;
const COLS: usize = 80;

// Some test semaphores
static mut current: (usize, usize) = (0,0);
static mut s1: StaticSemaphore = StaticSemaphore::new(1);
static mut s2: StaticSemaphore = StaticSemaphore::new(1);

// Some test routines
pub fn run(this: &Process) -> usize {
    let mut w0 = Window::new(COLS, ROWS, (0, 0));
    let mut msg = Window::new(60, 4, (1,1));

    unsafe { *(0xF000_0000 as *mut usize) = this.pid; }

    w0.set_bg(Color::LightBlue);
    w0.paint();

    msg.set_cursor((0,0));
    msg.set_bg(Color::LightGray);
    msg.set_fg(Color::Black);

    msg.put_str("<-- If semaphores work correctly, then only this block \
                should be red when all loop_procs finish running.");

    for _ in 0..206*3 {
        ready_queue::make_ready(Process::new("loop_proc", super::user::run2));
        unsafe { s1.down(); }

        // test vm
        if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
            panic!("Oh no! *0xF000_0000 should be {} but is {}",
                   this.pid, unsafe { *(0xF000_0000 as *mut usize) });
        }
    }

    msg.put_str("\n\nYay :)");

    0
}

fn get_prev((r,c): (usize, usize)) -> (usize, usize) {
    if r == 0 && c > 0 { // top
        (0, c-1)
    } else if c == COLS-1 { // right
        (r-1, c)
    } else if r == ROWS-1 { // bottom
        (r, c+1)
    } else { // left
        (r+1, c)
    }
}

fn get_next((r,c): (usize, usize)) -> (usize, usize) {
    if r == 0 && c < COLS-1 { // top
        (0, c+1)
    } else if c == COLS-1 && r < ROWS-1 { // right
        (r+1, c)
    } else if r == ROWS-1 && c > 0 { // bottom
        (r, c-1)
    } else { // left
        (r-1, c)
    }
}

fn run2(this: &Process) -> usize {

    unsafe { *(0xF000_0000 as *mut usize) = this.pid; }

    // test vm
    if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xF000_0000 should be {} but is {}",
               this.pid, unsafe { *(0xF000_0000 as *mut usize) });
    }

    unsafe { s2.down(); }

    let mut w = Window::new(COLS,ROWS, (0,0));

    let me = unsafe { current };
    let prev = get_prev(me);

    unsafe {
        current = get_next(me);
    }

    printf!("Erase ({},{}) ", prev.0, prev.1);
    w.set_bg(Color::LightBlue);
    w.set_cursor(prev);
    w.put_char(' ');

    printf!("Draw ({},{})\n", me.0,me.1);
    w.set_bg(Color::Red);
    w.set_cursor(me);
    w.put_char(' ');

    unsafe { s2.up(); }

    // test vm
    if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xF000_0000 should be {} but is {}",
               this.pid, unsafe { *(0xF000_0000 as *mut usize) });
    }

    unsafe { s1.up(); }

    0
}
