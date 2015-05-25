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
// TODO: exit
// TODO: kill
// TODO: test yielding to other q's

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

use self::context::{KContext};

mod current;
mod ready_queue;

mod context;

mod init;
mod user;

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

// Process struct
pub struct Process {
    name: &'static str,
    pid: usize,
    run: fn(&Process) -> usize,
    state: State,
    stack: usize, // pointer to stack, but *mut usize is not Send/Sync for some reason
    kcontext: KContext,
}

impl Process {
    pub fn new(name: &'static str, run: fn(&Process) -> usize) -> Process {

        let mut p = Process {
            name: name,
            pid: NEXT_ID.get_then_add(),
            run: run,
            state: State::INIT,
            stack: 0,
            kesp: 0,
        };

        p.get_stack();

        p
    }

    fn get_stack(&mut self) {
        // TODO: fudge
        // Allocate a stack
        let stack = Box::new([0; STACK_SIZE]);

        let stack_ptr = unsafe {boxed::into_raw(stack)} as *mut usize;

        // set the pointer
        self.stack = stack_ptr as usize;

        printf!("stack for {:?} is at 0x{:x}\n", self, self.stack);

        // smash the stack
        unsafe {
            for idx in (STACK_SIZE - 6)..STACK_SIZE {
                *stack_ptr.offset(idx as isize) = 0;
            }
            *stack_ptr.offset(STACK_SIZE as isize - 2) = start_proc as usize;
        }

        self.kesp = unsafe { stack_ptr.offset(STACK_SIZE as isize - 6) } as usize;
    }
}

impl Clone for Process {
    fn clone(&self) -> Process {
        printf!("clone process 0x{:X}\n", self as *const Process as usize);
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
    let code = match current::current() {
        Some(c_box) => { ((*c_box).run)(&*c_box) }
        None => { panic!("No current process to start") }
    };
    exit(code);
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

    printf!("{:?} yielding\n", me);

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
                    (*me_ptr).state = State::READY;
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

    // set the current proc
    current::set_current(ready_queue::ready_q().pop());

    // rebarrow next process to run
    let mut next = match current::current_mut() { 
        Some(n) => { n }
        None => { panic!("Nothing to do!") /* TODO: idle process */ }
    };

    //(*next).state = State::RUNNING;

    //// if this is the same process, do nothing
    //match me_rebarrowed {
    //    Some(ref me_r) => {
    //        if **me_r == *next {return;}
    //    }
    //    None => {}
    //}

    //// prepare to context switch
    //let current_kesp = match me_rebarrowed {
    //    Some(ref me_r) => (&(me_r.kesp as usize)) as *const usize,
    //    None => 0 as *const usize,
    //};
    //let next_kesp = next.kesp as usize; // get this value before moving next

    ////TODO : don't forget to change this when preemption is turned on
    //let eflags = 0; // if disableCount == 0 {(1<<9)} else {0}; // should we enable interupts?

    //printf!("looking for next_kesp as {:X}\n", next as *mut Box<Process> as usize);
    //printf!("context switch to {}, c={:x}, n={:x}, e={:x}\n", next.name, current_kesp as usize, next_kesp as usize, eflags as usize);

    //// then start running the process
    //unsafe {
    //    contextSwitch(current_kesp, next_kesp, eflags);
    //}

    // TODO: check killed

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
    proc_yield(None);
}
