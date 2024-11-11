#![no_std]
extern crate alloc;

pub mod args;
pub mod printer;
pub mod start;
pub mod test;

pub use rps2_libtest_macros::test;

pub use args::Args;
pub use test::{Test, TestBuilder};

#[doc(hidden)]
pub mod __hidden {
    pub extern crate inventory;

    use rps2_startup::Termination;

    #[inline(always)]
    pub fn test_invoke<T: Termination>(f: fn() -> T)
    where
        T: Termination,
    {
        match f().report() {
            0 => {}
            res => panic!("test ended with non-zero error code: {}", res),
        }
    }
}
