use alloc::boxed;
use alloc::boxed::Box;

use core::option::Option::{self, Some, None};

use super::concurrency::Atomic32;
use super::data_structures::Queue;

mod init;
mod current;
mod ready_queue;

// constants
const STACK_SIZE: usize = 2048;

// next pid
static NEXT_ID: Atomic32 = Atomic32 {i: 0};

// Process state
enum State {
    INIT,
    READY,
    RUNNING,
    BLOCKED,
    TERMINATED,
}

// A wrapper around stack pointers for use in initializing the stack
#[derive(Copy,Clone)]
struct StackPtr {
    ptr: *mut usize,
}

impl StackPtr {
    fn get_stack() -> StackPtr {
        // TODO: fudge
        // Allocate a stack
        let stack = Box::new([0; STACK_SIZE]);

        // set the pointer
        StackPtr {ptr: unsafe {boxed::into_raw(stack) as *mut usize} }
    }

    fn smash(&self) {
        // Get entry point of process
        let entry = start_proc as usize;

        // Smash the stack
        unsafe {
            for idx in (STACK_SIZE - 6)..STACK_SIZE {
                *self.ptr.offset(idx as isize) = 0;
            }
            *self.ptr.offset(-2) = entry;
        }
    }

    fn get_kesp(&self) -> StackPtr {
        // Create a new struct that contains the starting
        // kesp value for this stack
        StackPtr {ptr : (self.ptr as usize - 6) as *mut usize}
    }
}

// Process struct
pub struct Process {
    name: &'static str,
    pid: usize,
    run: fn(&Process) -> usize,
    state: State,
    stack: StackPtr,
    kesp: StackPtr,
}

impl Process {
    pub fn new(name: &'static str, run: fn(&Process) -> usize) -> Process {
        let stack = StackPtr::get_stack();
        stack.smash();

        Process {
            name: name,
            pid: NEXT_ID.get_then_add(),
            run: run,
            state: State::INIT,
            stack: stack,
            kesp: stack.get_kesp(),
        }
    }

    pub fn dispatch(prev: Option<Box<Process>>) {
        //TODO
    }
}

pub fn init() {

    // Create the init process
    let mut init = Box::new(Process::new("init", self::init::run));

    // Create the ready q
    ready_queue::init();

    // Add the init process to the ready q
    ready_queue::make_ready(init);

}

// The entry point of all processes
fn start_proc() {
    // get current process
    let c_op = current::current();
    match c_op {
        Some(c_box) => {
            ((*c_box).run)(&*c_box);
        }
        None => {
            panic!("Start proc without current process");
        }
    }
}

pub fn proc_yield(q: Option<Box<Queue<Box<Process>>>>) {

}
