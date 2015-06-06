// This is an implementation of simple cooperative processes
//
// Invariants:
// 1) Every process is owned by a box
// 2) Every process box is owned by
//      a) A queue
//      OR
//      b) The current process pointer
//
// TODO: check killed
// TODO: kill
// TODO: test yielding to other q's

use alloc::boxed;
use alloc::boxed::Box;

//use core::marker::{Sync, Send};
use core::option::Option::{self, Some, None};
use core::clone::Clone;
use core::fmt::{Debug, Formatter, Result};
use core::cmp::PartialEq;

use super::concurrency::Atomic32;
use super::data_structures::Queue;

use super::machine::{context_switch};
pub use super::machine::proc_yield;

use self::context::{KContext};

mod current;
mod ready_queue;

pub mod context;

mod init;
mod user;

// constants
const STACK_SIZE: usize = 2048;

// next pid
static NEXT_ID: Atomic32 = Atomic32 {i: 0};

// Process state
#[repr(C)]
#[derive(Clone, Copy)]
enum State {
    INIT,
    READY,
    RUNNING,
    BLOCKED,
    TERMINATED,
}

// Process struct
#[repr(packed)] // need this for box to work
pub struct Process {
    name: &'static str,
    pid: usize,
    run: fn(&Process) -> usize,
    state: State,
    stack: usize, // pointer to stack, but *mut usize is not Send/Sync for some reason
    kcontext: KContext,
}

impl Process {
    pub fn new(name: &'static str, run: fn(&Process) -> usize) -> Box<Process> {

        let mut p = Process {
            name: name,
            pid: NEXT_ID.get_then_add(),
            run: run,
            state: State::INIT,
            stack: 0,
            kcontext: KContext::new(),
        };

        p.get_stack();

        Box::new(p)
    }

    fn get_stack(&mut self) {
        // TODO: fudge
        // Allocate a stack
        let stack = Box::new([0; STACK_SIZE]);

        let stack_ptr = unsafe {boxed::into_raw(stack)} as *mut usize;

        // set the pointer
        self.stack = stack_ptr as usize;

        // smash the stack
        unsafe {
            // put RA on stack to return to start_proc
            *stack_ptr.offset(STACK_SIZE as isize - 2) = start_proc as usize;
        }

        self.kcontext.esp = unsafe { stack_ptr.offset(STACK_SIZE as isize - 2) } as usize;

        printf!("stack for {:?} is at 0x{:x}\n", self, self.stack);
    }
}

impl Clone for Process {
    fn clone(&self) -> Process {
        // printf!("clone process 0x{:X}\n", self as *const Process as usize);
        Process {
            name: self.name.clone(),
            pid: self.pid.clone(),
            run: self.run,
            state: self.state.clone(),
            stack: self.stack.clone(),
            kcontext: self.kcontext.clone(),
        }
    }
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Process #{}: {}", self.pid, self.name)
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Process) -> bool {
        // TODO: make this more comprehensive?
        self.pid == other.pid
    }
}

pub fn init() {
    // Init ready q and current proc
    current::init();
    ready_queue::init();

    // Create the init process
    let init = Process::new("init", self::init::run);

    // Add the init process to the ready q
    ready_queue::make_ready(init);
}

// The entry point of all processes
#[allow(private_no_mangle_fns)]
#[no_mangle]
fn start_proc() {
    // TODO check killed

    // run current process
    let code = match current::current() {
        Some(c_box) => {
            printf!("Starting {:?}\n", *c_box);
            ((*c_box).run)(&*c_box)
        }
        None => { panic!("No current process to start") }
    };
    exit(code);
}

// Restore the context of the next process on the ready q
// and switch to that process.
//
// This function should not be called in any process's code.
// It is called by the kernel to
// - yield
// - exit
// - handle syscalls
// - handle signals
pub fn switch_to_next() {
    // get next process from ready q
    let next = ready_queue::ready_q().pop();

    // set current process and context switch to it
    match next {
        Some(p) => {
            current::set_current(Some(p));
        }
        None => {
            // TODO: idle process
            panic!("Empty ready q");
        }
    }

    // context switch
    // TODO: eflags
    match current::current() {
        Some(p) => { unsafe{ context_switch(p.kcontext, 0); } }
        None => { panic!("No current to switch to!"); }
    }
}

// Yield to the next process waiting
//
// This function is called by the current process to
// yield to the next process on the ready q.
//
// Do not call this process directly! Call it through process::proc_yield()
#[no_mangle]
pub fn _proc_yield(q: Option<&mut Queue<Box<Process>>>) {

    // TODO: lock here

    // move current process to the ready q if there is one
    match current::current() {
        Some(c) => {
            ready_queue::make_ready(c);
            current::set_current(None);
        }
        None => { }
    }

    // switch to next process
    switch_to_next();

    //TODO: unlock here
}

// Called by the current process to exit
pub fn exit(code: usize) {
    match current::current_mut() {
        Some(current) => {
            if current.pid == 0 {
                panic!("{:?} is exiting!\n", current);
            } else {
                current.state = State::TERMINATED;
                printf!("{:?} exiting with code {}\n", current, code);
            }
        }
        None => panic!("Exiting with no current process!\n")
    }

    // set current to None, so we will never run this again
    current::set_current(None);

    // switch to next ready process
    switch_to_next();
}
