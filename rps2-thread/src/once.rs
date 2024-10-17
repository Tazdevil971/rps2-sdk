use crate::mutex::Mutex;
use core::fmt::{self, Debug};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Once {
    flag: AtomicBool,
    mutex: Mutex<()>,
}

impl Once {
    pub fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
            mutex: Mutex::new(()),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    pub fn call_once<F: FnOnce()>(&self, f: F) {
        if !self.is_completed() {
            self.call_once_inner(f);
        }
    }

    #[cold]
    fn call_once_inner<F: FnOnce()>(&self, f: F) {
        // Lock the internal mutex
        let _guard = self.mutex.lock();

        // Do a double check on flag
        if !self.flag.load(Ordering::Acquire) {
            f();
            self.flag.store(true, Ordering::Release);
        }
    }
}

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Once {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Once").finish_non_exhaustive()
    }
}
