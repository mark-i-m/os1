// This is an implementation of simple cooperative processes
//
// TODO: kill
// TODO: check killed

use alloc::boxed::Box;

use core::option::Option::{self, Some, None};
use core::fmt::{Debug, Formatter, Result};
use core::cmp::PartialEq;

use core::sync::atomic::{AtomicUsize, Ordering};

use core::ops::Drop;

use super::data_structures::ProcessQueue;

use super::interrupts::{on, off};
use super::interrupts::pit::JIFFIES;

use super::machine::{self, context_switch};

use super::memory::{AddressSpace, esp0};

use self::context::{KContext};

use self::idle::IDLE_PROCESS;

pub use self::current::{CURRENT_PROCESS};

pub mod context;
pub mod current;

pub mod ready_queue;

mod init;
mod idle;
mod reaper;
mod user;

// constants
const STACK_SIZE: usize = 2048;

// next pid
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

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
pub struct Process {
    // Name of the process; some string
    name: &'static str,
    pid: usize,

    // This is the code of the process
    run: fn(&Process) -> usize,

    state: State,

    // The kheap-allocated stack
    // - this is a pointer to the stack, but *mut usize is not Send/Sync
    // - this pointer is to the bottom of the stack, not the head
    stack: usize,

    // The saved kernel context for context switching
    kcontext: KContext,

    // Address space
    pub addr_space: AddressSpace,

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
            pid: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            run: run,
            state: State::INIT,
            stack: 0,
            kcontext: KContext::new(),
            addr_space: AddressSpace::new(),
            disable_cnt: 0,
            next_proc: 0 as *mut Process,
        };

        p.get_stack();

        Box::into_raw(box p)
    }

    fn get_stack(&mut self) {
        // TODO: fudge
        // Allocate a stack
        let stack: Box<[usize; STACK_SIZE]> = box [0; STACK_SIZE];

        let stack_ptr = Box::into_raw(stack) as *mut usize;

        // set the pointer
        self.stack = stack_ptr as usize;

        // smash the stack
        unsafe {
            // put RA on stack to return to start_proc
            *stack_ptr.offset(STACK_SIZE as isize - 2) = start_proc as usize;
        }

        self.kcontext.esp = unsafe { stack_ptr.offset(STACK_SIZE as isize - 2) } as usize;

        // printf!("stack for {:?} is at 0x{:x}\n", self, self.stack);
    }

    fn set_state(&mut self, s: State) {
        self.state = s;
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.stack as *mut [usize; 2048]);
        }
        // let the box go out of scope to dealloc
    }
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Process #{} @ 0x{:x}: {}",
               self.pid, self as *const Process as usize, self.name)
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Process) -> bool {
        self.pid == other.pid
    }
}

pub fn init() {
    // Init ready q and current proc
    current::init(); // THIS MUST BE FIRST
    ready_queue::init();

    // Add the init process to the ready q
    ready_queue::make_ready(Process::new("init", self::init::run));

    // Create the idle process
    idle::init();

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
            printf!("{:?} [Starting]\n", *CURRENT_PROCESS);
            (*CURRENT_PROCESS).set_state(State::RUNNING);
            ((*CURRENT_PROCESS).run)(&*CURRENT_PROCESS)
        } else {
            panic!("No current process to start")
        };

        exit(code);
    }
}

// A safe wrapper around machine::proc_yield
pub fn proc_yield<'a>(q: Option<&'a mut ProcessQueue>) {
    unsafe {
        off();
        machine::proc_yield(q);
        on();
    }
}

// Yield to the next process waiting
//
// This function is called by the current process to
// yield to the next process on the ready q.
//
// To yield normally, call process::proc_yield()
//
// Interrupts should already be disabled here
#[no_mangle]
#[inline(never)]
pub unsafe fn _proc_yield<'a>(q: Option<&'a mut ProcessQueue>) {
    // yield onto the right queue
    if !CURRENT_PROCESS.is_null() {
        if let Some(queue) = q {
            (*CURRENT_PROCESS).set_state(State::BLOCKED);
            queue.push_tail(CURRENT_PROCESS);
        } else {
            ready_queue::make_ready(CURRENT_PROCESS);
        }

        CURRENT_PROCESS = 0 as *mut Process;
    }

    // get next process from ready q
    // every 50 jiffies, spawn a reaper process
    let mut next = if JIFFIES % 50 == 0 {
        Process::new("reaper", self::reaper::run)
    } else {
        ready_queue::get_next()
    };

    // run the idle process when everyone is done
    if next.is_null() {
        next = IDLE_PROCESS;
    }

    //bootlog!("{:?} [Switching]\n", *CURRENT_PROCESS);

    // switch address spaces
    (*next).addr_space.activate();

    // switch stacks
    esp0((*CURRENT_PROCESS).stack + STACK_SIZE*4);

    // set the CURRENT_PROCESS
    CURRENT_PROCESS = next;

    // context switch
    context_switch((*CURRENT_PROCESS).kcontext,
                   if (*CURRENT_PROCESS).disable_cnt == 0 {
                       1 << 9
                   } else {
                       0
                   });

    panic!("The impossible has happened!");
}

// Called by the current process to exit
pub fn exit(code: usize) {
    // Disable interrupts
    off();

    unsafe {
        if !CURRENT_PROCESS.is_null() {
            // check if the init process is exiting
            if (*CURRENT_PROCESS).pid == 0 {
                //panic!("{:?} is exiting!\n", *CURRENT_PROCESS);
            } else {
                (*CURRENT_PROCESS).set_state(State::TERMINATED);
                printf!("{:?} [Exit 0x{:X}]\n",
                        *CURRENT_PROCESS, code);
            }

            // reaper
            self::reaper::reaper_add(CURRENT_PROCESS);
        } else {
            panic!("Exiting with no current process!\n");
        }

        // set current to None, so we will never run this again
        CURRENT_PROCESS = 0 as *mut Process;

        // switch to next ready process
        _proc_yield(None);
    }

    panic!("The impossible has happened!");
}
