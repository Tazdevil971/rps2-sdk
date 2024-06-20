#![no_std]
extern crate alloc;

pub mod args;
pub mod entry;
pub mod printer;
pub mod test;

pub use rps2_libtest_macros::test;

pub use args::Args;
pub use test::{Test, TestBuilder};

#[doc(hidden)]
pub mod __hidden {
    pub extern crate inventory;
}
