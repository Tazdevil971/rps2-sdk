use crate::mutex::Mutex;
use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::mem::MaybeUninit;
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

pub struct OnceLock<T> {
    once: Once,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceLock<T> {
    pub fn new() -> OnceLock<T> {
        Self {
            once: Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            Some(unsafe {
                // SAFETY: We just checked that the value is in fact initialized
                self.get_unchecked()
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            Some(unsafe {
                // SAFETY: We just checked that the value is in fact initialized
                self.value.get_mut().assume_init_mut()
            })
        } else {
            None
        }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.is_initialized() {
            // Reset the once
            self.once = Once::new();

            // Read the previous value
            Some(unsafe {
                // SAFETY: We just checked that the value is in fact initialized
                self.value.get_mut().assume_init_read()
            })
        } else {
            None
        }
    }

    pub fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> &T {
        self.init(f);
        unsafe {
            // SAFETY: After calling init the value is guaranteed to be initialized
            self.get_unchecked()
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        let mut value = Some(value);
        self.init(|| value.take().unwrap());

        if let Some(value) = value {
            Err(value)
        } else {
            Ok(())
        }
    }

    fn init<F: FnOnce() -> T>(&self, f: F) {
        self.once.call_once(move || {
            let value = f();
            unsafe {
                // SAFETY: Only one thread can enter this section at one, and the value is not yet
                // signaled to be initialized, so we are guaranteed to be the only ones having
                // access to the data
                (*self.value.get()).write(value);
            }
        });
    }

    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        (*self.value.get()).assume_init_ref()
    }
}

impl<T> Drop for OnceLock<T> {
    fn drop(&mut self) {
        if self.is_initialized() {
            unsafe {
                // SAFETY: We just checked that the value is in fact initialized
                self.value.get_mut().assume_init_drop();
            }
        }
    }
}

impl<T> Default for OnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for OnceLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("OnceLock");
        match self.get() {
            Some(value) => d.field(&value),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

impl<T: Clone> Clone for OnceLock<T> {
    fn clone(&self) -> Self {
        let cell = Self::new();
        if let Some(value) = self.get() {
            let _ = cell.set(value.clone());
        }
        cell
    }
}
