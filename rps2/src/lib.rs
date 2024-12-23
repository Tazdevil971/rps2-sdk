#![no_std]
extern crate alloc as alloc_crate;
extern crate rps2_startup;

pub use rps2_kernel::{dbg, interrupt_disable_guard, kprint, kprintln};

pub mod prelude {
    pub use crate::boxed::Box;
    pub use crate::string::String;
    pub use crate::vec::Vec;
    pub use crate::{dbg, kprint, kprintln};
}

pub mod sync {
    pub use alloc_crate::sync::*;
    pub use core::sync::*;

    pub use rps2_thread::lazy_lock::LazyLock;
    pub use rps2_thread::mutex::{IrqMutexGuard, Mutex, MutexGuard};
    pub use rps2_thread::once::Once;
    pub use rps2_thread::once_lock::OnceLock;
    pub use rps2_thread::sema::{Sema, SemaBuilder};

    pub mod mpmc {
        pub use rps2_thread::mpmc::{BoundedQueue, UnboundedQueue};
    }
}

pub mod thread {
    pub use rps2_thread::thread::{
        current, panicking, rotate_ready_queue, sleep, spawn, Builder, JoinHandle, Result, Thread,
    };

    pub mod ffi {
        pub use rps2_thread::ffi::*;
    }
}

pub mod arch {
    pub use rps2_kernel::arch::*;
}

pub mod os {
    pub use rps2_kernel::os::*;

    pub mod deci2 {
        pub use rps2_kernel::deci2::*;
    }
}

pub mod alloc {
    pub use alloc_crate::alloc::*;
}

pub mod collections {
    pub use alloc_crate::collections::*;
}

pub mod boxed {
    pub use alloc_crate::boxed::*;
}

pub mod string {
    pub use alloc_crate::string::*;
}

pub mod vec {
    pub use alloc_crate::vec::*;
}

pub mod panic {
    pub use core::panic::*;
    pub use rps2_panic::{abort, catch_unwind, panic_any, panicking, set_backtrace_enabled};
}
