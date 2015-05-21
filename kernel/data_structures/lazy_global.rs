// Written by Krzysztof Drewniak krzysz00/rust-kernel

#![allow(dead_code)]

use core::marker::{Sync, Send};
use core::cell::UnsafeCell;
use core::option::Option::{self, Some, None};

pub struct LazyGlobal<T> {
    pub data: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send> Send for LazyGlobal<T> { }
unsafe impl<T: Send> Sync for LazyGlobal<T> { }

impl<T> LazyGlobal<T> {
    // You must call this before using the global
    pub unsafe fn init(&self, val: T) {
        *self.data.get() = Some(val);
    }

    pub unsafe fn get<'a>(&'a self) -> &'a T {
        match *self.data.get() {
            Some(ref val) => val,
            None => panic!("Lazy global not initialized")
        }
    }

    pub unsafe fn get_mut<'a>(&'a self) -> &'a mut T {
        match *self.data.get() {
            Some(ref mut val) => val,
            None => panic!("Lazy global not initialized")
        }
    }

    pub unsafe fn unwrap(self) -> T {
        match self.data.into_inner() {
            Some(val) => val,
            None => panic!("Lazy global not initialized")
        }
    }
}

#[macro_export]
macro_rules! lazy_global {
    () => (
        $crate::data_structures::LazyGlobal {
            data: ::core::cell::UnsafeCell { value: ::core::option::Option::None }
        });
    ($ty:ty) => (
        $crate::data_structures::LazyGlobal<$ty> {
            data: ::core::cell::UnsafeCell<$ty> { value: ::core::option::Option::None }
        });
}
