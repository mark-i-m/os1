//! A simple doubly-linked list implementation

/// A linked list implementation.
/// - O(1) push to head/tail
/// - O(1) pop from head/tail
/// - O(1) access to head/tail
/// - Automatic memory management
pub struct LinkedList<T> {
    head: *mut Link<T>,
    tail: *mut Link<T>,
    size: usize,
}

/// A link in the `LinkedList`
struct Link<T> {
    val: T,
    next: *mut Link<T>,
}

impl<T> LinkedList<T> {
    /// Create a new empty List
    pub fn new() -> LinkedList<T> {
        // TODO
    }

    /// Push to the end of the list
    pub fn push_back(&mut self, val: T) {
        // TODO
    }

    /// Pop and return tail of list
    pub fn pop_back(&mut self) -> T {
        // TODO
    }

    /// Return an immutable reference to the tail
    pub fn back(&self) -> &T {
        // TODO
    }

    /// Return a mutable reference to the tail
    pub fn back_mut(&mut self) -> &mut T {
        // TODO
    }

    /// Push to the front of the list
    pub fn push_fron(&mut self, val: T) {
        // TODO
    }

    /// Pop and return head of list
    pub fn pop_front(&mut self) -> T {
        // TODO
    }

    /// Return an immutable reference to the head
    pub fn front(&self) -> &T {
        // TODO
    }

    /// Return a mutable reference to the head
    pub fn front_mut(&mut self) -> &mut T {
        // TODO
    }

    /// Return the number of elements
    pub fn size(&self) -> usize {
        self.len
    }
}

impl<T> Link<T> {
    /// Create a new link containing the given value
    fn new(val: T) -> Link<T> {
        Link {
            val: val,
            next: 0 as *mut Link<T>,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        // TODO
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // TODO
    }
}
