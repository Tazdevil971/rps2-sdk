use crate::once::Once;
use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::mem::ManuallyDrop;
use core::ops::Deref;

union Data<T, F> {
    pub value: ManuallyDrop<T>,
    pub f: ManuallyDrop<F>,
}

pub struct LazyLock<T, F = fn() -> T> {
    once: Once,
    data: UnsafeCell<Data<T, F>>,
}

impl<T, F> LazyLock<T, F> {
    pub fn new(f: F) -> Self {
        Self {
            once: Once::new(),
            data: UnsafeCell::new(Data {
                f: ManuallyDrop::new(f),
            }),
        }
    }

    pub fn into_inner(this: Self) -> Result<T, F> {
        // Prevent destruction of this object
        let mut this = ManuallyDrop::new(this);

        if this.once.is_completed() {
            Ok(unsafe { ManuallyDrop::take(&mut this.data.get_mut().value) })
        } else {
            Err(unsafe { ManuallyDrop::take(&mut this.data.get_mut().f) })
        }
    }

    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.once.is_completed());
        unsafe { &(*self.data.get()).value }
    }

    fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    pub fn force(this: &Self) -> &T {
        this.once.call_once(|| {
            // SAFETY: Only one thread can enter this section at one, and the value is not yet
            // signaled to be initialized, so we are guaranteed to be the only ones having
            // access to the data
            let data = unsafe { &mut *this.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });

        // SAFETY: Now the data is guaranteed to be initialized
        unsafe { this.get_unchecked() }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::force(self)
    }
}

impl<T: Default> Default for LazyLock<T> {
    fn default() -> Self {
        Self::new(T::default)
    }
}

impl<T, F> Drop for LazyLock<T, F> {
    fn drop(&mut self) {
        if self.once.is_completed() {
            unsafe {
                ManuallyDrop::drop(&mut self.data.get_mut().value);
            }
        } else {
            unsafe {
                ManuallyDrop::drop(&mut self.data.get_mut().f);
            }
        }
    }
}

impl<T: Debug> Debug for LazyLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_tuple("LazyLock");
        match self.get() {
            Some(value) => {
                d.field(value);
            }
            None => {
                d.field(&format_args!("<uninit>"));
            }
        }
        d.finish()
    }
}

unsafe impl<T: Sync + Send, F: Send> Sync for LazyLock<T, F> {}
