use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::fmt::{self, Debug};
use critical_section::Mutex;

use crate::sema::Sema;

struct SyncDequeue<T> {
    inner: Mutex<RefCell<VecDeque<T>>>,
}

impl<T> SyncDequeue<T> {
    fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(VecDeque::new())),
        }
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(VecDeque::with_capacity(cap))),
        }
    }

    fn push(&self, val: T) {
        critical_section::with(|cs| {
            self.inner.borrow_ref_mut(cs).push_back(val);
        });
    }

    fn pop(&self) -> Option<T> {
        critical_section::with(|cs| self.inner.borrow_ref_mut(cs).pop_front())
    }
}

pub struct UnboundedQueue<T> {
    inner: SyncDequeue<T>,
    csema: Sema,
}

impl<T> UnboundedQueue<T> {
    pub fn new() -> Self {
        let csema = Sema::builder()
            .init_count(0)
            .max_count(i32::MAX as u32)
            .build()
            .expect("Failed to build sema");

        Self {
            inner: SyncDequeue::new(),
            csema,
        }
    }

    pub fn push(&self, val: T) {
        self.inner.push(val);
        self.csema.signal();
    }

    pub fn pop(&self) -> T {
        self.csema.wait();
        let res = self.inner.pop();
        res.expect("Queue is actually empty!")
    }

    pub fn try_pop(&self) -> Option<T> {
        if self.csema.poll() {
            let res = self.inner.pop();
            Some(res.expect("Queue is actually empty!"))
        } else {
            None
        }
    }

    pub unsafe fn irq_push(&self, val: T) {
        self.inner.push(val);
        self.csema.irq_signal();
    }

    pub unsafe fn irq_try_pop(&self) -> Option<T> {
        if self.csema.irq_poll() {
            let res = self.inner.pop();
            Some(res.expect("Queue is actually empty!"))
        } else {
            None
        }
    }
}

impl<T> Default for UnboundedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for UnboundedQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnboundedQueue").finish_non_exhaustive()
    }
}

pub struct BoundedQueue<T> {
    inner: SyncDequeue<T>,
    csema: Sema,
    psema: Sema,
}

impl<T> BoundedQueue<T> {
    pub fn new(count: usize) -> Self {
        let csema = Sema::builder()
            .init_count(0)
            .max_count(count as _)
            .build()
            .expect("Failed to build sema");

        let psema = Sema::builder()
            .init_count(count as _)
            .max_count(count as _)
            .build()
            .expect("Failed to build sema");

        Self {
            inner: SyncDequeue::with_capacity(count),
            csema,
            psema,
        }
    }

    pub fn push(&self, val: T) {
        self.psema.wait();
        self.inner.push(val);
        self.csema.signal();
    }

    pub fn try_push(&self, val: T) -> Result<(), T> {
        if self.psema.poll() {
            self.inner.push(val);
            self.csema.signal();
            Ok(())
        } else {
            Err(val)
        }
    }

    pub fn pop(&self) -> T {
        self.csema.wait();
        let res = self.inner.pop();
        let res = res.expect("Queue is actually empty!");
        self.psema.signal();
        res
    }

    pub fn try_pop(&self) -> Option<T> {
        if self.csema.poll() {
            let res = self.inner.pop();
            let res = res.expect("Queue is actually empty!");
            self.psema.signal();
            Some(res)
        } else {
            None
        }
    }

    pub unsafe fn irq_try_push(&self, val: T) -> Result<(), T> {
        if self.psema.irq_poll() {
            self.inner.push(val);
            self.csema.irq_signal();
            Ok(())
        } else {
            Err(val)
        }
    }

    pub unsafe fn irq_try_pop(&self) -> Option<T> {
        if self.csema.irq_poll() {
            let res = self.inner.pop();
            let res = res.expect("Queue is actually empty!");
            self.psema.irq_signal();
            Some(res)
        } else {
            None
        }
    }
}

impl<T> Debug for BoundedQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BoundedQueue").finish_non_exhaustive()
    }
}
