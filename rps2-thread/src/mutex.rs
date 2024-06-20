use crate::sema::Sema;

use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub struct Mutex<T: ?Sized> {
    sema: Sema,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    // Make the type !Send + !Sync
    _marker: PhantomData<*const ()>,
}

pub struct IrqMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    // Make the type !Send + !Sync
    _marker: PhantomData<*const ()>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        let sema = Sema::builder()
            .init_count(1)
            .max_count(1)
            .build()
            .expect("Failed to create semaphore");

        Self {
            data: UnsafeCell::new(val),
            sema,
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.sema.wait();
        MutexGuard {
            lock: self,
            _marker: PhantomData,
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.sema.poll() {
            Some(MutexGuard {
                lock: self,
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    pub unsafe fn irq_try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.sema.irq_poll() {
            Some(MutexGuard {
                lock: self,
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Debug + ?Sized> Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Mutex").finish_non_exhaustive()
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.data.get().as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.data.get().as_mut().unwrap() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.sema.signal();
    }
}

impl<T: Debug + ?Sized> Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> Deref for IrqMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.data.get().as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for IrqMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.data.get().as_mut().unwrap() }
    }
}

impl<T: ?Sized> Drop for IrqMutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            self.lock.sema.irq_signal();
        }
    }
}

impl<T: Debug + ?Sized> Debug for IrqMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}
