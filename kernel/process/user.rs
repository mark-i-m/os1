//! A module for user processes
// So far it just contains some test code

use core::ptr;

use super::super::concurrency::{Semaphore, StaticSemaphore};
use super::super::vga::window::{Window, Color};
use super::Process;
use super::ready_queue;
use super::proc_table::PROCESS_TABLE;

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
    let mut msg_sem = unsafe {
        let s = 0xD000_0000 as *mut Semaphore<Window>;
        ptr::write(s, Semaphore::new(Window::new(60, 7, (1,1)), 1));
        &mut *s
    };

    unsafe { *(0xF000_0000 as *mut usize) = this.pid; }

    w0.set_bg(Color::LightBlue);
    w0.paint();

    let mut msg = msg_sem.down();
    msg.set_cursor((0,0));
    msg.set_bg(Color::LightGray);
    msg.set_fg(Color::Black);

    msg.put_str("<-- If semaphores work correctly, then only this block \
                should be red when all loop_procs finish running.");

    for i in 0..206*3 {
        ready_queue::make_ready(Process::new("loop_proc", self::run2));
        unsafe { s1.down(); }

        // test vm
        if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
            panic!("Oh no! *0xF000_0000 should be {} but is {}",
                   this.pid, unsafe { *(0xF000_0000 as *mut usize) });
        }
    }

    msg.put_str("\n\nYay! :)");

    msg.put_str("\n\nNow test semaphores... ");

    // TODO: uncomment again
    //unsafe {
    //    // create another process
    //    let p = Process::new("semaphore_test", self::run3);
    //    ready_queue::make_ready(p);

    //    // share the semaphore
    //    (*p).addr_space.request_share(this.pid, 0xD000_0000);
    //}

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

    // test process table
    unsafe {
        let me = PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!");

        if (me as *const Process) != (this as *const Process) {
            panic!("Oh no! me = 0x{:X}, but should be 0x{:X}",
                   me as usize, this as *const Process as usize);
        }
    }

    unsafe { s1.up(); }

    0
}

fn run3(this: &Process) -> usize {
    unsafe {
        let me = &mut *PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!");

        // accept share from the parent process
        while !me.addr_space.accept_share(3, 0xF000_0000) { }

        let mut msg_sem = {
            let s = 0xF000_0000 as *mut Semaphore<Window>;
            *s = Semaphore::new(Window::new(60, 4, (1,1)), 1);
            &mut *s
        };

        let mut msg = msg_sem.down();

        msg.put_str("Success! :D");
    }

    0
}
