// A module containing useful data structures

pub use self::queue::Queue;
pub use self::linked_list::LinkedList;
pub use self::lazy_global::LazyGlobal;

mod queue;
mod linked_list;

#[macro_use]
mod lazy_global;

pub mod concurrency;
