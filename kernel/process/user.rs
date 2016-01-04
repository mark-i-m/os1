//! A module for user processes
// So far it just contains some test code

use core::fmt::Write;
use core::ptr;

use concurrency::{Semaphore, StaticSemaphore};
use io::stream::InputStream;
use string::String;
use vga::rectangle::{Rectangle, Color};
use super::Process;
use super::ready_queue;
use super::focus::focus;
use super::proc_table::PROCESS_TABLE;

// useful constants
const ROWS: usize = 25;
const COLS: usize = 80;
const NUM_LOOP: usize = (ROWS*2 + COLS*2 - 4) * 3;

// Some test semaphores
static mut current: (usize, usize) = (0,0);
static mut s1: StaticSemaphore = StaticSemaphore::new(1);
static mut s2: StaticSemaphore = StaticSemaphore::new(1);

// Some test routines
pub fn run(this: &Process) -> usize {

    // grab focus
    focus(None);

    let mut w0 = Rectangle::new(COLS, ROWS, (0, 0));
    let mut msg_sem = unsafe {
        let s = 0xD000_0000 as *mut Semaphore<Rectangle>;
        ptr::write(s, Semaphore::new(Rectangle::new(60, 7, (1,1)), 1));
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

    for _ in 0..NUM_LOOP {
        ready_queue::make_ready(Process::new("loop_proc", self::run2));
        unsafe { s1.down(); }

        // test vm
        if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
            panic!("Oh no! *0xF000_0000 should be {} but is {}",
                   this.pid, unsafe { *(0xF000_0000 as *mut usize) });
        }
    }

    msg.put_str("\n\nYay! :)");

    // test share-page IPC
    unsafe {
        // create another process
        ready_queue::make_ready(Process::new("semaphore_test", self::run3));

        // share the semaphore
        let me = &mut *PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!");
        if !me.addr_space.request_share(this.pid + NUM_LOOP + 1, 0xD000_0000) {
            panic!("Share request failed!");
        }
    }

    msg.put_str("\n\nNow test IPC... ");

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

    let mut w = Rectangle::new(COLS,ROWS, (0,0));

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
        if !me.addr_space.accept_share(3, 0xF000_0000) {
            panic!("Share accept failed");
        }

        let mut msg_sem = {
            let s = 0xF000_0000 as *mut Semaphore<Rectangle>;
            &mut *s
        };

        let mut msg = msg_sem.down();

        msg.put_str("Success! :D");
    }

    ready_queue::make_ready(Process::new("kbd_proc", self::run4));

    0
}

fn run4(this: &Process) -> usize {
    // test the keyboard

    // get focus
    focus(None);

    // accept kbd input
    let me = unsafe {
        &mut *PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!")
    };
    me.accept_kbd(3);

    // accept input until we get a \n
    let mut w0 = Rectangle::new(40, 4, (8, 1));
    w0.set_bg(Color::Green);
    w0.set_fg(Color::Red);
    w0.paint();

    let mut string = String::new();

    let buffer = me.buffer.as_mut().expect("Oh no! Why isn't there a buffer?");
    loop {
        match buffer.next() {
            Some('\n') => break,
            Some('\x08') => { // backspace
                let _ = string.pop();
                if string.len() == 0 {
                    w0.paint();
                } else {
                    w0.set_cursor((0,0));
                    write!(&mut w0, "{}", string);
                }
            }
            Some(c) => {
                string.push(c);
                w0.set_cursor((0,0));
                write!(&mut w0, "{}", string);
            },
            None => {},
        }
    }

    0
}
