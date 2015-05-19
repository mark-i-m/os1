// Implementing cooperative processes before turning on preemption

use core::marker::{Sized, Sync};
use alloc::boxed;
use alloc::boxed::{into_raw, Box};
use super::data_structures::{Queue};

pub use self::init::{Init};

pub mod init;

// constants
const STACK_SIZE: usize = 2048;

// The currently running process
// access through process::current() and process::set_current()
use self::Static::*;

#[allow(dead_code)]
enum Static<T> {
    There(T),
    Null,
}

#[allow(dead_code)]
static mut current_proc: Static<*mut Process> = Null;
static mut ready_queue: Static<*mut Queue<Box<Process>>> = Null;

// This is the API that all processes must implement
//
// note: if methods does not take &self, then Process is not object-safe
pub trait Process : Sync {
    fn new(name: &'static str) -> Self
        where Self : Sized;
    fn run(&self) -> usize;
}

pub enum State {
    READY,
    RUNNING,
    BLOCKED,
    TERMINATED,
}

struct StackPtr {
    ptr: *const usize,
}

unsafe impl Sync for StackPtr { }

impl StackPtr {
    fn get_stack() -> StackPtr {
        // Allocate a stack
        let stack = Box::new([0; STACK_SIZE]);

        // set the pointer
        StackPtr {ptr: unsafe {boxed::into_raw(stack) as *const usize} }
    }

    fn smash(&self) {
        // Smash the stack
    }

    fn get_kesp(&self) -> StackPtr {
        // Create a new struct that contains the starting
        // kesp value for this stack
    }
}

// This function initializes the process infrastructure
pub fn init() {
    printf!("In process init\n");

    // create the ready queue
    unsafe {
        ready_queue = There(boxed::into_raw(Box::new(Queue::new())));
    }

    // create Init process, but do not start it
    let init: Box<Process> = Box::new(Init::new("Init"));

    // add to ready queue
    make_ready(init);

    printf!("Created init process\n");
}

// get the current process as a Box
#[allow(dead_code)]
pub fn current() -> Box<Process> {
    // TODO: lock here
    unsafe {
        match current_proc {
            There(p) => {
                Box::from_raw(p)
            }
            Null   => {
                panic!("No current process");
            }
        }
    }
    // TODO: lock here
}

// set the current process from a Box
#[allow(dead_code)]
pub fn set_current(process: Box<Process>) {

    // TODO: lock here

    unsafe {
        current_proc = There(boxed::into_raw(process));
    }

    // TODO: unlock here
}

// Yield to the next process waiting
pub fn proc_yield() {

}

// Add the process to the Ready queue
pub fn make_ready(process: Box<Process>) {

    // TODO: lock here

    unsafe {
        match ready_queue {
            There(q) => {
                let mut rq: Box<Queue<Box<Process>>> = Box::from_raw(q);
                (*rq).push(process);
            }
            Null => {
                panic!("Unitialized ready queue");
            }
        }
    }

    // TODO: unlock here
}

// Atomically get the next id
pub fn next_id() -> usize {
    0//TODO
}
