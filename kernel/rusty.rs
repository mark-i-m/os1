#![feature(no_std)]
#![feature(lang_items)]

#![no_std]

pub mod rusty{

    // Copied from <http://doc.rust-lang.org/core/marker/trait.PhantomFn.html>
    #[lang="phantom_fn"]
    pub trait PhantomFn<A: ?Sized, R: ?Sized = ()> {}

    #[lang="sized"]
    pub trait Sized: PhantomFn<Self> {}

    #[lang="copy"]
    pub trait Copy: PhantomFn<Self> {}

    #[lang="sync"]
    pub trait Sync: PhantomFn<Self> {}

}
