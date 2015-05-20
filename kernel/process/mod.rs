// This is an implementation of simple cooperative processes
//
// Invariants:
// 1) Every process is owned by a box
// 2) Every process box is owned by
//      a) A queue
//      OR
//      b) The current process pointer
// 3) All queues are owned by boxes

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

    pub fn dispatch(&mut self, prev: Option<&Box<Process>>) {
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
pub fn proc_yield(mut q: Option<Box<Queue<Box<Process>>>>) {

    // TODO: lock here

    let mut me = current::current();
    let mut me_barrow = None; 

    match me {
        Some(mut me_ptr) => {
            match q {
                Some(mut q_ptr) => {
                    /* a queue is specified, I'm blocking on that queue */
                    //if (me_ptr->iDepth != 0) {
                    //    Debug::printf("process %s#%d %X ", me_ptr->name, me_ptr->id, me_ptr);
                    //    Debug::panic("blocking while iDepth = %d",me_ptr->iDepth);
                    //}
                    (*me_ptr).state = State::BLOCKED;
                    (*q_ptr).push(me_ptr);

                    // Debug::printf("blocking process %s#%d %X\n", me->name, me->id, me);
                }
                None => {
                    /* no queue is specified, put me on the ready queue */
                    ready_queue::make_ready(me_ptr);
                }
            }
            me_barrow = ready_queue::barrow_current();
        }
        _ => {}
    }

    let mut next: Box<Process>;
    let mut rq = ready_queue::ready_q();

    if ((*rq).is_empty()) {
        panic!("Nothing to do!");
        //if (!idleProcess) {
        //    idleProcess = new IdleProcess();
        //    idleProcess->start();
        //}
        //next = idleProcess;
    } else {
        match rq.pop() {
            Some(n) => { next = n; }
            None    => { panic!("ready q is pseudo-empty"); }
        }
    }

    (*next).dispatch(me_barrow);

    //TODO: unlock here
}
