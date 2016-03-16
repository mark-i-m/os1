//! A module for process management

use alloc::boxed::Box;

use core::fmt::{Debug, Formatter, Result};
use core::sync::atomic::{AtomicUsize, Ordering};

use interrupts::{on, off};
use io::NonBlockingBuffer;
use machine::{self, context_switch};
use memory::{AddressSpace, esp0};
use static_linked_list::StaticLinkedList;

use self::context::KContext;
use self::idle::IDLE_PROCESS;
use self::proc_table::PROCESS_TABLE;

pub mod context;
pub mod focus;
pub mod proc_table;
pub mod ready_queue;

mod idle;
mod init;
mod reaper;
mod user;

/// Size of a kernel stack (number of words)
const STACK_SIZE: usize = 2048;

/// The next available PID
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// The current running process if there is one
pub static mut CURRENT_PROCESS: *mut Process = 0 as *mut Process;

/// Type alias for a queue of Processes
pub type ProcessQueue = StaticLinkedList<*mut Process>;

/// An enum representing the possible states of a process
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    /// Process is created, but not ready
    INIT,
    /// Process is in the ready q
    READY,
    /// Process is running (not on the ready q or blocked)
    RUNNING,
    /// Process is blocked on a q
    BLOCKED,
    /// Process has died
    TERMINATED,
}

/// Represents a single process, its identity, resources, etc.
pub struct Process {
    /// Name of the process; not necessarily unique
    name: &'static str,

    /// Unique 32-bit identifier
    pid: usize,

    /// The routine of the process
    run: fn(&Process) -> usize,

    /// The current state of the process
    state: State,

    /// A pointer to the kheap-allocated stack space for this process's
    /// kernel stack. This pointer is to the bottom of the stack, not the head.
    stack: usize,

    /// The saved kernel context for context switching
    kcontext: KContext,

    /// The virtual memory address space of the process
    pub addr_space: AddressSpace,

    /// Number of calls to interrupts::on() while this process was running
    /// Interrupts are on if `disable_cnt == 0`
    pub disable_cnt: usize,

    /// A keyboard input buffer
    pub buffer: Option<NonBlockingBuffer>,

    /// Current working file (inode number)
    pub cwf: usize, 
}

impl Process {
    /// Create a new process with the given name and routine. Because processes
    /// are a fundamental abstraction, they are too low level for me to use Rust well.
    /// For this reason, a raw pointer is returned to the process, and it is the
    /// job of the caller to arrange for the process to be reaped.
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
            buffer: None,
            cwf: 0,
        };

        p.get_stack();

        let process = Box::into_raw(box p);

        unsafe {
            PROCESS_TABLE.push(process);
        }

        process
    }

    /// A helper to get a kernel stack for this process
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

    /// Set the state of the process to `s`
    fn set_state(&mut self, s: State) {
        self.state = s;
    }

    /// Get the state of the process
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Get the PID of this process
    pub fn get_pid(&self) -> usize {
        self.pid
    }

    /// Start accepting keyboard input when this process
    /// gains focussed. The buffer will have the capacity
    /// given.
    pub fn accept_kbd(&mut self, cap: usize) {
        if self.buffer.is_none() {
            self.buffer = Some(NonBlockingBuffer::new(cap));
        }
    }

    // TODO: implement pwd (pwf) and cd (cf)
}

impl Drop for Process {
    /// When the process is reaped, we must free its stack
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.stack as *mut [usize; 2048]);
        }
        // let the box go out of scope to dealloc
    }
}

impl Debug for Process {
    /// Allow processes to be printed elegantly in format strings
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Process #{} @ 0x{:x}: {}",
               self.pid, self as *const Process as usize, self.name)
    }
}

impl PartialEq for Process {
    /// Two processes are the same if they have the same PID
    fn eq(&self, other: &Process) -> bool {
        self.pid == other.pid
    }
}

/// Initialize the process subsystem.
/// This creates the init, idle, and reaper processes, but does not
/// start any of them yet.
pub fn init() {
    // Add the init process to the ready q
    ready_queue::make_ready(Process::new("init", self::init::run));

    // Create the idle process
    idle::init();

    // Create the reaper process
    reaper::init();

    printf!("processes inited\n");
}

/// The entry point of all processes
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

/// A safe wrapper around machine::proc_yield.
///
/// This function is called when the current process wants to
/// yield the rest of its quantum. The process can specify a
/// `ProcessQueue` to yield onto, or if None is specified, it
/// yields onto the ready queue.
pub fn proc_yield<'a>(q: Option<&'a mut ProcessQueue>) {
    unsafe {
        off();
        machine::proc_yield(q);
        on();
    }
}

/// The unsafe function that does the actual work of choosing the
/// next process and switching to it.
///
/// NOTE: Processes should not call this function directly; they
/// should use `process::proc_yield` instead.
/// NOTE: Interrupts should already be disabled here
#[no_mangle]
#[inline(never)]
pub unsafe fn _proc_yield<'a>(q: Option<&'a mut ProcessQueue>) {
    // yield onto the right queue
    if !CURRENT_PROCESS.is_null() {
        if let Some(queue) = q {
            (*CURRENT_PROCESS).set_state(State::BLOCKED);
            //bootlog!("{:?} [Blocking]\n", *CURRENT_PROCESS);
            queue.push_back(CURRENT_PROCESS);
        } else {
            ready_queue::make_ready(CURRENT_PROCESS);
        }

        CURRENT_PROCESS = 0 as *mut Process;
    }

    // get next process from ready q
    let mut next = ready_queue::get_next();

    // run the idle process when everyone is done
    if next.is_null() {
        next = IDLE_PROCESS;
    }

    // switch address spaces
    (*next).addr_space.activate();

    // switch stacks
    // TODO: turn this on
    //esp0((*CURRENT_PROCESS).stack + STACK_SIZE*4);

    // set the CURRENT_PROCESS
    CURRENT_PROCESS = next;

    //bootlog!("{:?} [Switching]\n", *CURRENT_PROCESS);

    // context switch
    context_switch((*CURRENT_PROCESS).kcontext,
                   if (*CURRENT_PROCESS).disable_cnt == 0 {
                       1 << 9
                   } else {
                       0
                   });

    panic!("The impossible has happened!");
}

/// Called by the current process to exit with the given exit code
pub fn exit(code: usize) {
    unsafe {
        if CURRENT_PROCESS.is_null() {
            panic!("Exiting with no current process!\n");
        }

        (*CURRENT_PROCESS).set_state(State::TERMINATED);

        // clean up address space
        (*CURRENT_PROCESS).addr_space.clear();

        // Disable interrupts
        off();

        // NOTE: need to print *before* adding to reaper q
        bootlog!("{:?} [Exit 0x{:X}]\n", *CURRENT_PROCESS, code);

        self::reaper::reaper_add(CURRENT_PROCESS);

        // set current to None, so we will never run this again
        CURRENT_PROCESS = 0 as *mut Process;

        // switch to next ready process
        _proc_yield(None);
    }

    panic!("The impossible has happened!");
}
