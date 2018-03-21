//! A module for user processes
//! So far it just contains some test code

use core::fmt::Write;
use core::ptr;

use io::block::BlockDataBuffer;
use io::stream::InputStream;
use fs::ROOT_FS;
use sync::{Semaphore, StaticSemaphore};
use vga::rectangle::{Rectangle, Color};
use vga::input::{InputElement, TextBox};
use super::Process;
use super::focus::focus;
use super::ready_queue;
use super::proc_table::PROCESS_TABLE;

// useful constants
const ROWS: usize = 25;
const COLS: usize = 80;
const NUM_LOOP: usize = (ROWS * 2 + COLS * 2 - 4) * 3;

// Some test semaphores
static mut CURRENT: (usize, usize) = (0, 0);
static mut S1: StaticSemaphore = StaticSemaphore::new(1);
static mut S2: StaticSemaphore = StaticSemaphore::new(1);

// Some test routines
pub fn run(this: &Process) -> usize {

    // grab focus
    focus(None);

    let mut w0 = Rectangle::new(COLS, ROWS, (0, 0));
    let msg_sem = unsafe {
        let s = 0xD000_0000 as *mut Semaphore<Rectangle>;
        ptr::write(s, Semaphore::new(Rectangle::new(60, 7, (1, 1)), 1));
        &*s
    };

    unsafe {
        *(0xF000_0000 as *mut usize) = this.pid;
    }

    w0.set_bg(Color::LightBlue);
    w0.paint();

    let mut msg = msg_sem.down();
    msg.set_cursor((0, 0));
    msg.set_bg(Color::LightGray);
    msg.set_fg(Color::Black);

    msg.put_str("<-- If semaphores work correctly, then only this block \
                should be red when all loop_procs finish running.");

    for _ in 0..NUM_LOOP {
        ready_queue::make_ready(Process::new("loop_proc", self::run2));
        unsafe {
            S1.down();
        }

        // test vm
        if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
            panic!("Oh no! *0xF000_0000 should be {} but is {}",
                   this.pid,
                   unsafe { *(0xF000_0000 as *mut usize) });
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

fn get_prev((r, c): (usize, usize)) -> (usize, usize) {
    if r == 0 && c > 0 {
        // top
        (0, c - 1)
    } else if c == COLS - 1 {
        // right
        (r - 1, c)
    } else if r == ROWS - 1 {
        // bottom
        (r, c + 1)
    } else {
        // left
        (r + 1, c)
    }
}

fn get_next((r, c): (usize, usize)) -> (usize, usize) {
    if r == 0 && c < COLS - 1 {
        // top
        (0, c + 1)
    } else if c == COLS - 1 && r < ROWS - 1 {
        // right
        (r + 1, c)
    } else if r == ROWS - 1 && c > 0 {
        // bottom
        (r, c - 1)
    } else {
        // left
        (r - 1, c)
    }
}

fn run2(this: &Process) -> usize {

    unsafe {
        *(0xF000_0000 as *mut usize) = this.pid;
    }

    // test vm
    if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xF000_0000 should be {} but is {}",
               this.pid,
               unsafe { *(0xF000_0000 as *mut usize) });
    }

    unsafe {
        S2.down();
    }

    let mut w = Rectangle::new(COLS, ROWS, (0, 0));

    let me = unsafe { CURRENT };
    let prev = get_prev(me);

    unsafe {
        CURRENT = get_next(me);
    }

    printf!("Erase ({},{}) ", prev.0, prev.1);
    w.set_bg(Color::LightBlue);
    w.set_cursor(prev);
    w.put_char(' ');

    printf!("Draw ({},{})\n", me.0, me.1);
    w.set_bg(Color::Red);
    w.set_cursor(me);
    w.put_char(' ');

    unsafe {
        S2.up();
    }

    // test vm
    if unsafe { *(0xF000_0000 as *mut usize) } != this.pid {
        panic!("Oh no! *0xF000_0000 should be {} but is {}",
               this.pid,
               unsafe { *(0xF000_0000 as *mut usize) });
    }

    // test process table
    unsafe {
        let me = PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!");

        if (me as *const Process) != (this as *const Process) {
            panic!("Oh no! me = 0x{:X}, but should be 0x{:X}",
                   me as usize,
                   this as *const Process as usize);
        }
    }

    unsafe {
        S1.up();
    }

    0
}

fn run3(this: &Process) -> usize {
    unsafe {
        let me = &mut *PROCESS_TABLE.get(this.pid).expect("Oh no! expected Some(process)!");

        // accept share from the parent process
        if !me.addr_space.accept_share(3, 0xF000_0000) {
            panic!("Share accept failed");
        }

        let msg_sem = {
            let s = 0xF000_0000 as *mut Semaphore<Rectangle>;
            &*s
        };

        let mut msg = msg_sem.down();

        msg.put_str("Success! :D");
    }

    ready_queue::make_ready(Process::new("kbd_proc", self::run4));

    0
}

fn run4(_: &Process) -> usize {
    // test the keyboard

    let mut label = Rectangle::new(17, 1, (8, 1));
    label.set_bg(Color::Black);
    label.set_fg(Color::Yellow);
    label.set_cursor((0, 0));
    label.put_str("Enter your name: ");

    let mut input_box = TextBox::new(40, (8, 18));
    let string = input_box.get_str();

    let mut b = Rectangle::new(70, 10, (10, 1));
    b.set_bg(Color::LightGray);
    b.set_fg(Color::Black);
    b.set_cursor((0, 0));

    let _ = write!(&mut b, "Hello, {}!\n", string);

    ready_queue::make_ready(Process::new("fs_proc", self::run5));

    0
}

fn run5(_: &Process) -> usize {
    // test the fs

    let mut f = unsafe { (*ROOT_FS).open_read(0).ok().unwrap() };

    let mut buf = BlockDataBuffer::new(512);

    f.seek(512);

    f.read(&mut buf);
    let val3 = unsafe { *buf.get_ref::<usize>(0) };

    f.seek(0);

    buf.set_offset(0);
    f.read(&mut buf);
    let val1 = unsafe { *buf.get_ref::<usize>(0) };
    let val2 = unsafe {
        (*buf.get_ref::<usize>(126) & 0xFFFF_0000) | (*buf.get_ref::<usize>(127) & 0x0000_FFFF)
    };

    let mut b = Rectangle::new(70, 10, (12, 1));
    b.set_bg(Color::LightGray);
    b.set_fg(Color::Black);
    b.set_cursor((0, 0));

    let _ = write!(&mut b,
                   "Read values from file {}: {:8X}, {:8X}, {:8X} ... ",
                   f.get_filename(),
                   val1,
                   val2,
                   val3);
    if val1 == 0xDEADBEEF && val2 == 0xDEADBEB0 && val3 == 0xDEADBEBE {
        let _ = write!(&mut b, "Success!\n");
    } else {
        let _ = write!(&mut b, "Failure :[");
    }

    f.seek(516);

    let mut buf2 = BlockDataBuffer::new(32);
    // unsafe {
    //    *buf2.get_ref_mut::<usize>(0) = 0xCAFEBABE;
    //    *buf2.get_ref_mut::<usize>(2) = 0xCAFEBABE;
    //    *buf2.get_ref_mut::<usize>(4) = 0xCAFEBABE;
    //    *buf2.get_ref_mut::<usize>(6) = 0xCAFEBABE;
    //    *buf2.get_ref_mut::<usize>(1) = 0xBAADFACE;
    //    *buf2.get_ref_mut::<usize>(3) = 0xBAADFACE;
    //    *buf2.get_ref_mut::<usize>(5) = 0xBAADFACE;
    //    *buf2.get_ref_mut::<usize>(7) = 0xBAADFACE;
    // }

    // f.write(32, &mut buf2);

    f.seek(32);
    buf2.set_offset(0);
    f.read(&mut buf2);
    // buf2.set_offset(0);

    // let btct = unsafe { *buf2.get_ref::<usize>(0) };
    let version_major = unsafe { *buf2.get_ref::<usize>(0) };
    let version_minor = unsafe { *buf2.get_ref::<usize>(1) };

    printf!("v{}.{}", version_major, version_minor);

    // unsafe { *buf2.get_ref_mut::<usize>(0) = btct+1 };
    //
    // f.seek(32);
    // f.write(4, &mut buf2);

    // let _ = write!(&mut b, "\nBoot #{}\n", btct);

    let mut v = Rectangle::new(15, 1, (24, 64));
    v.set_bg(Color::LightBlue);
    v.set_fg(Color::Red);
    v.set_cursor((0, 0));
    let _ = write!(&mut v, "os1 v{}.{}", version_major, version_minor);

    0
}
