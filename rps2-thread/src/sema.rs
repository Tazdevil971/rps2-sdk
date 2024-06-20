use core::mem;

use crate::{ffi, Result};

#[derive(Debug, Clone, Copy)]
pub struct SemaBuilder {
    max_count: u32,
    init_count: u32,
}

impl SemaBuilder {
    pub fn new() -> Self {
        Self {
            max_count: 255,
            init_count: 0,
        }
    }

    pub fn max_count(mut self, count: u32) -> Self {
        self.max_count = count;
        self
    }

    pub fn init_count(mut self, count: u32) -> Self {
        self.init_count = count;
        self
    }

    pub fn build(self) -> Result<Sema> {
        // Clamp these values
        let max_count = self.max_count.min(i32::MAX as _);
        let init_count = self.init_count.min(i32::MAX as _);

        unsafe {
            ffi::create_sema(ffi::SemaParam {
                count: 0,
                max_count: max_count as _,
                init_count: init_count as _,
                attr: 0,
                option: 0,
            })
            .map(Sema)
        }
    }
}

impl Default for SemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Sema(i32);

impl Sema {
    pub fn into_raw(sema: Sema) -> i32 {
        let tid = sema.0;
        // Prevent Drop from running
        mem::forget(sema);
        tid
    }

    pub unsafe fn from_raw(sid: i32) -> Self {
        Self(sid)
    }

    pub fn builder() -> SemaBuilder {
        SemaBuilder::default()
    }

    pub fn new() -> Result<Self> {
        SemaBuilder::default().build()
    }

    pub fn id(&self) -> i32 {
        self.0
    }

    pub fn wait(&self) {
        unsafe {
            ffi::wait_sema(self.0).expect("Semaphore got unexpectedly deleted!");
        }
    }

    pub fn signal(&self) {
        unsafe {
            ffi::signal_sema(self.0).expect("Semaphore got unexpectedly deleted!");
        }
    }

    pub fn poll(&self) -> bool {
        unsafe { ffi::poll_sema(self.0).is_ok() }
    }

    pub fn delete(self) {
        unsafe {
            ffi::delete_sema(Self::into_raw(self)).expect("Semaphore got unexpectedly deleted!");
        }
    }

    pub unsafe fn irq_signal(&self) {
        ffi::irq_signal_sema(self.0).expect("Semaphore got unexpectedly deleted!");
    }

    pub unsafe fn irq_poll(&self) -> bool {
        ffi::irq_poll_sema(self.0).is_ok()
    }

    pub unsafe fn irq_delete(self) {
        ffi::irq_delete_sema(Self::into_raw(self)).expect("Semaphore got unexpectedly deleted!");
    }
}

impl Drop for Sema {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_sema(self.0).expect("Semaphore got unexpectedly deleted!");
        }
    }
}
