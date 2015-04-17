// Minimal libcore

#![feature(no_std)]
#![feature(lang_items)]

#![no_std]

#![allow(dead_code)]
#![allow(unused_attributes)]

// Copied from <http://doc.rust-lang.org/core/marker/trait.PhantomFn.html>
#[lang="phantom_fn"]
pub trait PhantomFn<A: ?Sized, R: ?Sized = ()> {}

#[lang="sized"]
pub trait Sized: PhantomFn<Self> {}

#[lang="copy"]
pub trait Copy: PhantomFn<Self> {}

#[lang="sync"]
pub trait Sync: PhantomFn<Self> {}

pub enum Option<T> {
    None,
    Some(T)
}

pub struct IntRange {
    cur: i32,
    max: i32
}

impl IntRange {
    fn next(&mut self) -> Option<i32> {
        if self.cur < self.max {
            self.cur += 1;
            Option::Some(self.cur - 1)
        } else {
            Option::None
        }
    }
}
