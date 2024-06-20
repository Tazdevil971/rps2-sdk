#![no_std]
extern crate alloc;
pub extern crate unwinding;

use core::any::Any;
use core::cell::RefCell;
use core::ffi::c_void;
use core::panic::{Location, PanicInfo};
use core::sync::atomic::{AtomicBool, Ordering};

use alloc::boxed::Box;
use alloc::string::ToString;

use critical_section::Mutex;
use smallvec::SmallVec;

#[cfg(feature = "unwinding")]
use unwinding::abi::*;

static BACKTRACE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_backtrace_enabled(value: bool) {
    BACKTRACE_ENABLED.store(value, Ordering::SeqCst);
}

pub fn abort() -> ! {
    unsafe {
        rps2_kernel::os::exit(-1);
    }
}

#[track_caller]
pub fn panic_any<M: 'static + Any + Send>(msg: M) -> ! {
    rps2_kernel::kprintln!("panicked at {}", Location::caller());
    begin_unwind(Box::new(msg));
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    rps2_kernel::kprintln!("{}", info);

    let msg = info.message();
    if let Some(msg) = msg.as_str() {
        begin_unwind(Box::new(msg));
    } else {
        begin_unwind(Box::new(msg.to_string()));
    }
}

pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    #[cfg(feature = "unwinding")]
    {
        let res = unwinding::panic::catch_unwind(f);
        if res.is_err() {
            // The current thread exited panicking state
            tracker::unset_panicking();
        }

        res
    }

    #[cfg(not(feature = "unwinding"))]
    {
        Ok(f())
    }
}

pub fn panicking() -> bool {
    #[cfg(feature = "unwinding")]
    {
        tracker::panicking()
    }
    #[cfg(not(feature = "unwinding"))]
    {
        false
    }
}

fn begin_unwind(msg: Box<dyn Any + Send>) -> ! {
    #[cfg(feature = "unwinding")]
    {
        // Insert into panicking list
        if !tracker::set_panicking() {
            if BACKTRACE_ENABLED.load(Ordering::SeqCst) {
                trace::print();
            }

            rps2_kernel::kprintln!("thread panicked while processing panic. aborting.");
            abort();
        }

        // Print out a backtrace
        if BACKTRACE_ENABLED.load(Ordering::SeqCst) {
            trace::print();
        }

        // Actually begin panicking
        let code = unwinding::panic::begin_panic(msg);
        rps2_kernel::kprintln!("failed to initiate panic, error {}", code.0);
        abort();
    }

    #[cfg(not(feature = "unwinding"))]
    {
        // Instantly abort as it's not supported
        abort();
    }
}

#[cfg(feature = "unwinding")]
mod trace {
    use super::*;

    pub fn print() {
        let mut i = 0;
        trace(|ip| {
            i += 1;
            rps2_kernel::kprintln!("{i:4}:{ip:#12x} - <unknown>");
        });
    }

    pub fn trace<F: FnMut(usize)>(mut f: F) {
        extern "C" fn callback<F: FnMut(usize)>(
            ctx: &UnwindContext<'_>,
            arg: *mut c_void,
        ) -> UnwindReasonCode {
            let f = arg as *mut F;
            let ip = _Unwind_GetIP(ctx);
            unsafe {
                (*f)(ip as usize);
            }
            UnwindReasonCode::NO_REASON
        }

        _Unwind_Backtrace(callback::<F>, &mut f as *mut F as *mut c_void);
    }
}

#[cfg(feature = "unwinding")]
mod tracker {
    use super::*;

    // List of threads currently panicking
    static THREADS: Mutex<RefCell<SmallVec<[i32; 128]>>> =
        Mutex::new(RefCell::new(SmallVec::new_const()));

    fn threads_search(tid: i32) -> bool {
        critical_section::with(|cs| {
            let threads = THREADS.borrow_ref(cs);

            threads.iter().copied().find(|tid2| *tid2 == tid).is_some()
        })
    }

    fn threads_insert(tid: i32) -> bool {
        critical_section::with(|cs| {
            let mut threads = THREADS.borrow_ref_mut(cs);

            let exists = threads.iter().copied().find(|tid2| *tid2 == tid).is_some();

            if !exists {
                threads.push(tid);
                true
            } else {
                false
            }
        })
    }

    fn threads_remove(tid: i32) -> bool {
        critical_section::with(|cs| {
            let mut threads = THREADS.borrow_ref_mut(cs);

            let position = threads.iter().copied().position(|tid2| tid2 == tid);

            if let Some(position) = position {
                threads.swap_remove(position);
                true
            } else {
                false
            }
        })
    }

    pub fn set_panicking() -> bool {
        let tid = unsafe { rps2_kernel::os::get_thread_id() };
        threads_insert(tid)
    }

    pub fn unset_panicking() {
        let tid = unsafe { rps2_kernel::os::get_thread_id() };
        threads_remove(tid);
    }

    pub fn panicking() -> bool {
        let tid = unsafe { rps2_kernel::os::get_thread_id() };
        threads_search(tid)
    }
}
