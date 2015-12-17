// Will eventually be a user process, but there is a long way to go
use super::Process;
use super::ready_queue;

use super::super::vga::window::{Window, Color};

use super::super::data_structures::concurrency::{StaticSemaphore};

const ROWS: usize = 25;
const COLS: usize = 80;

static mut current: (usize, usize) = (0,0);
static mut s1: StaticSemaphore = StaticSemaphore::new(1);
static mut s2: StaticSemaphore = StaticSemaphore::new(1);

#[allow(unused_variables)]
pub fn run(this: &Process) -> usize {
    let mut w0 = Window::new(COLS, ROWS, (0, 0));
    let mut msg = Window::new(43, 4, (1,1));

    unsafe { *(0xf00000 as *mut usize) = this.pid; }

    w0.set_bg_color(Color::LightBlue);
    w0.paint();

    msg.set_cursor((0,0));
    msg.set_bg_color(Color::LightGray);
    msg.set_fg_color(Color::Black);

    msg.put_str("<-- If semaphores work correctly, then only this block \
                should be red when all loop_procs finish running");

    for _ in 0..206*3 {
        ready_queue::make_ready(Process::new("loop_proc", super::user::run2));
        unsafe { s1.down(); }

        // test vm
        if unsafe { *(0xf00000 as *mut usize) } != this.pid {
            panic!("Oh no! *0xf00000 should be {} but is {}",
                   this.pid, unsafe { *(0xf00000 as *mut usize) });
        }
    }

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

#[allow(unused_variables)]
fn run2(this: &Process) -> usize {

    unsafe { *(0xf00000 as *mut usize) = this.pid; }

    // test vm
    if unsafe { *(0xf00000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xf00000 should be {} but is {}",
               this.pid, unsafe { *(0xf00000 as *mut usize) });
    }

    unsafe { s2.down(); }

    let mut w = Window::new(COLS,ROWS, (0,0));

    let me = unsafe { current };
    let prev = get_prev(me);

    unsafe {
        current = get_next(me);
    }

    printf!("Erase ({},{}) ", prev.0, prev.1);
    w.set_bg_color(Color::LightBlue);
    w.set_cursor(prev);
    w.put_str(" ");

    printf!("Draw ({},{})\n", me.0,me.1);
    w.set_bg_color(Color::Red);
    w.set_cursor(me);
    w.put_str(" ");

    unsafe { s2.up(); }

    // test vm
    if unsafe { *(0xf00000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xf00000 should be {} but is {}",
               this.pid, unsafe { *(0xf00000 as *mut usize) });
    }

    unsafe { s1.up(); }

    0
}
