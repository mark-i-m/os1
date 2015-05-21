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

    pub fn dispatch(&mut self, prev: Option<&Box<Process>>) {
        //TODO
        (self.run)(self);
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

pub fn init() {

    // Init ready q and current proc
    current::init();
    ready_queue::init();
    printf!("created ready_q\n");

    // Create the init process
    let init = Box::new(Process::new("init", self::init::run));
    printf!("created init\n");

    // Add the init process to the ready q
    ready_queue::make_ready(init);
    printf!("queued init\n");

    let rq = ready_queue::ready_q();
    printf!("size {}\n", rq.size());
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

    printf!("yield\n");
    // TODO: lock here

    let mut me = current::current_mut(); // barrow the current proc
    let mut me: Option<&mut Box<Process>>  = None;

    match me {
        Some(ref mut me_ptr) => {
            printf!("Yuck\n");
            match q {
                Some(q_ptr) => {
                    /* a queue is specified, I'm blocking on that queue */
                    //if (me_ptr->iDepth != 0) {
                    //    Debug::printf("process %s#%d %X ", me_ptr->name, me_ptr->id, me_ptr);
                    //    Debug::panic("blocking while iDepth = %d",me_ptr->iDepth);
                    //}
                    (*me_ptr).state = State::BLOCKED;
                    (*q_ptr).push(match current::current() {
                        Some(cp) => cp,
                        None => panic!("Somebody has poisoned the waterhole!"),
                    });

                    // Debug::printf("blocking process %s#%d %X\n", me->name, me->id, me);
                }
                None => {
                    /* no queue is specified, put me on the ready queue */
                    ready_queue::make_ready(match current::current() {
                        Some(cp) => cp,
                        None => panic!("Somebody has poisoned the waterhole!"),
                    });
                }
            }
        }
        _ => {
            printf!("Hi\n");
        }
    }

    (*match (*ready_queue::ready_q()).pop() {
        Some(n) => { n }
        None => { panic!("Nothing to do!") }
    }).dispatch(match me {
        Some(cp) => Some(&*cp),
        None => None,
    });

    //TODO: unlock here
}
