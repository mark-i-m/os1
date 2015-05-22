// This is an implementation of simple cooperative processes
//
// Invariants:
// 1) Every process is owned by a box
// 2) Every process box is owned by
//      a) A queue
//      OR
//      b) The current process pointer

use alloc::boxed;
use alloc::boxed::Box;

use core::marker::{Sync, Send};
use core::option::Option::{self, Some, None};
use core::clone::Clone;
use core::fmt::{Debug, Formatter, Result};
use core::cmp::PartialEq;

use super::concurrency::Atomic32;
use super::data_structures::Queue;

use super::machine::{contextSwitch};

mod init;
mod current;
mod ready_queue;

// constants
const STACK_SIZE: usize = 2048;

// next pid
static NEXT_ID: Atomic32 = Atomic32 {i: 0};

// Process state
#[derive(Clone, Copy)]
enum State {
    INIT,
    READY,
    RUNNING,
    BLOCKED,
    TERMINATED,
}

// A wrapper around stack pointers for use in initializing the stack
#[allow(raw_pointer_derive)]
#[derive(Copy,Clone)]
struct StackPtr {
    ptr: *mut usize,
}

unsafe impl Send for StackPtr {}
unsafe impl Sync for StackPtr {}

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
            *self.ptr.offset(STACK_SIZE as isize - 2) = entry;
        }
    }

    fn get_kesp(&self) -> StackPtr {
        // Create a new struct that contains the starting
        // kesp value for this stack
        StackPtr {ptr : unsafe { self.ptr.offset(STACK_SIZE as isize - 6) } as *mut usize}
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
}

impl Clone for Process {
    fn clone(&self) -> Process {
        Process {
            name: self.name.clone(),
            pid: self.pid.clone(),
            run: self.run,
            state: self.state.clone(),
            stack: self.stack.clone(),
            kesp: self.kesp.clone(),
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
    let init = Box::new(Process::new("init", self::init::run));

    // Add the init process to the ready q
    ready_queue::make_ready(init);
}

// The entry point of all processes
#[allow(private_no_mangle_fns)]
#[no_mangle]
fn start_proc() {
    // get current process
    match current::current() {
        Some(c_box) => { ((*c_box).run)(&*c_box); }
        None => { panic!("No current process to start"); }
    }
}

// Yield to the next process waiting
//
// The current process yields onto the q
// If q is null, yield to ready q
// If current proc is null, then it is not added to the q
// The process state is set accordingly
//
// The next process from the ready q is dispatched
// If the q is empty, then the current process continues running
//
// Ownership:
// - Move ownership of current process to the queue
// - Lend the current process to the next process to save context
pub fn proc_yield(q: Option<&mut Queue<Box<Process>>>) {

    // TODO: lock here

    let mut me = current::current_mut(); // barrow the current proc
    let mut me_rebarrowed = None;

    match me {
        Some(ref mut me_ptr) => {
            match q {
                Some(q_ptr) => {
                    /* a queue is specified, I'm blocking on that queue */
                    //if (me_ptr->iDepth != 0) {
                    //    Debug::printf("process %s#%d %X ", me_ptr->name, me_ptr->id, me_ptr);
                    //    Debug::panic("blocking while iDepth = %d",me_ptr->iDepth);
                    //}
                    (*me_ptr).state = State::BLOCKED;
                    q_ptr.push(match current::current() {
                        Some(cp) => cp,
                        None => panic!("Somebody has poisoned the waterhole!"),
                    });
                    me_rebarrowed = q_ptr.peek_tail();

                    // Debug::printf("blocking process %s#%d %X\n", me->name, me->id, me);
                }
                None => {
                    /* no queue is specified, put me on the ready queue */
                    ready_queue::make_ready(match current::current() {
                        Some(cp) => cp,
                        None => panic!("Somebody has poisoned the waterhole!"),
                    });
                    me_rebarrowed = ready_queue::ready_q().peek_tail();
                }
            }
        }
        _ => { }
    }

    // next process to run
    let mut next = match ready_queue::ready_q().pop() {
        Some(n) => { n }
        None => { panic!("Nothing to do!") /* TODO: idle process */ }
    };

    (*next).state = State::RUNNING;

    // if this is the same process, do nothing
    match me_rebarrowed {
        Some(me_r) => {
            if *me_r == next {return;}
        }
        None => {}
    }

    // prepare to context switch
    let current_kesp = match me_rebarrowed {
        Some(me_r) => me_r.kesp.ptr,
        None => 0 as *mut usize,
    };
    let next_kesp = next.kesp.ptr as usize; // get this value before moving next

    //TODO : don't forget to change this when preemption is turned on
    let eflags = 0; // if disableCount == 0 {(1<<9)} else {0}; // should we enable interupts?

    // set the current proc
    current::set_current(Some(next));

    // then start running the process
    unsafe {
        contextSwitch(current_kesp, next_kesp, eflags);
    }

    // TODO: check killed

    //TODO: unlock here
}
