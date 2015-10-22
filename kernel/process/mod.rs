// This is an implementation of simple cooperative processes
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

use super::data_structures::concurrency::Atomic32;
use super::data_structures::ProcessQueue;

use super::interrupts::{on, off};

use super::machine::{context_switch};

use self::context::{KContext};

pub use self::current::{CURRENT_PROCESS};

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
#[repr(C,packed)]
pub struct Process {
    // Name of the process; some string
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

    // A link to the next process to be scheduled
    pub next_proc: *mut Process,
}

impl Process {
    pub fn new(name: &'static str, run: fn(&Process) -> usize) -> *mut Process {

        let mut p = Process {
            name: name,
            pid: NEXT_ID.get_then_add(),
            run: run,
            state: State::INIT,
            stack: 0,
            kcontext: KContext::new(),
            disable_cnt: 0,
            next_proc: 0 as *mut Process,
        };

        p.get_stack();

        unsafe { boxed::into_raw(box p) }
    }

    fn get_stack(&mut self) {
        // TODO: fudge
        // Allocate a stack
        let stack: Box<[usize; STACK_SIZE]> = box [0; STACK_SIZE];

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

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Process #{} @ 0x{:x}: {}", self.pid, self as *const Process as usize, self.name)
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
    current::init(); // THIS MUST BE FIRST
    ready_queue::init();

    // Add the init process to the ready q
    ready_queue::make_ready(Process::new("init", self::init::run));

    printf!("Processes inited\n");
}

// The entry point of all processes
#[allow(private_no_mangle_fns)]
#[no_mangle]
fn start_proc() {
    // TODO check killed

    // run current process
    unsafe {
        let code = if !CURRENT_PROCESS.is_null() {
            printf!("Starting {:?}\n", *CURRENT_PROCESS);
            ((*CURRENT_PROCESS).run)(&*CURRENT_PROCESS)
        } else {
            panic!("No current process to start")
        };

        exit(code);
    }
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
    // disable interrupts
    off();

    // get next process from ready q
    let next = ready_queue::get_next();

    // set current process and context switch to it
    unsafe {
        if !next.is_null() {
            CURRENT_PROCESS = next;
        } else {
            // TODO: idle process
            panic!("Empty ready q");
        }

        // context switch
        if !CURRENT_PROCESS.is_null() {
            context_switch((*CURRENT_PROCESS).kcontext,
                           if (*CURRENT_PROCESS).disable_cnt == 0 {
                               1 << 9
                           } else {
                               0
                           });
        } else {
            panic!("No current to switch to!");
        }
    }

    // enable interrupts
    on();
}

// safe wrapper around machine::proc_yield
pub fn proc_yield(q: Option<usize>) {
    // disable interrupts
    off();

    unsafe {super::machine::proc_yield(q)}

    // enable interrupts
    on();
}

// Yield to the next process waiting
//
// This function is called by the current process to
// yield to the next process on the ready q.
//
// Do not call this process directly! Call it through process::proc_yield()
//
// Interrupts should already be disabled here
#[no_mangle]
pub fn _proc_yield(q: Option<usize>) {
    // move current process to the ready q if there is one
    // TODO: yielding onto q

    unsafe {
        if !CURRENT_PROCESS.is_null() {
            ready_queue::make_ready(CURRENT_PROCESS);
            CURRENT_PROCESS = 0 as *mut Process;
        }
    }

    // switch to next process
    switch_to_next();
}

// Called by the current process to exit
pub fn exit(code: usize) {
    unsafe {
        if !CURRENT_PROCESS.is_null() {
            if (*CURRENT_PROCESS).pid == 0 {
                panic!("{:?} is exiting!\n", *CURRENT_PROCESS);
            } else {
                (*CURRENT_PROCESS).state = State::TERMINATED;
                printf!("{:?} exiting with code 0x{:X}\n",
                        CURRENT_PROCESS, code);
            }
        } else {
            panic!("Exiting with no current process!\n");
        }

        // set current to None, so we will never run this again
        CURRENT_PROCESS = 0 as *mut Process;
    }

    // switch to next ready process
    switch_to_next();
}
