#![feature(no_std)]
#![feature(lang_items)]

#![no_std]

// Copied from <http://doc.rust-lang.org/core/marker/trait.PhantomFn.html>
#[lang="phantom_fn"]
trait PhantomFn<A: ?Sized, R: ?Sized = ()> {}

#[lang="sized"]
trait Sized: PhantomFn<Self> {}

#[lang="copy"]
trait Copy: PhantomFn<Self> {}

#[lang="sync"]
trait Sync: PhantomFn<Self> {}


#[no_mangle]
#[no_stack_check]
pub fn kernel_main() {

}
