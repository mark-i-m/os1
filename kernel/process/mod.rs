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

use super::data_structures::Queue;
use super::data_structures::concurrency::Atomic32;

use super::machine::{context_switch};

use self::context::{KContext};

pub mod context;
pub mod current;

mod ready_queue;

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
#[repr(packed)]
pub struct Process {
    name: &'static str,
    pid: usize,

    // This is the code of the process
    run: fn(&Process) -> usize,

    state: State,

    // The kheap-allocated stack
    // - this is a pointer to the stack, but *mut usize is not Send/Sync for sone reason
    // - this pointer is to the bottom of the stack, not the head
    stack: usize,

    // The saved kernel context for context switching
    kcontext: KContext,

    // Number of calls to interrupts::on() while this process was running
    // Interrupts are on if disable_cnt == 0
    pub disable_cnt: usize,
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
            disable_cnt: 0,
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

    fn set_state(&mut self, s: State) {
        self.state = s;
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
            disable_cnt: self.disable_cnt.clone(),
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

    printf!("Processes inited\n");
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
        Some(p) => { unsafe{
            context_switch(p.kcontext,
                           if p.disable_cnt == 0 { 1 << 9 } else { 0 });
        } }
        None => { panic!("No current to switch to!"); }
    }
}

// safe wrapper around machine::proc_yield
pub fn proc_yield(q: Option<&mut Queue<Box<Process>>>) {
    unsafe {super::machine::proc_yield(q)}
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
    // TODO: yielding onto q
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
                printf!("{:?} exiting with code 0x{:X}\n", current, code);
            }
        }
        None => panic!("Exiting with no current process!\n")
    }

    // set current to None, so we will never run this again
    current::set_current(None);

    // switch to next ready process
    switch_to_next();
}
