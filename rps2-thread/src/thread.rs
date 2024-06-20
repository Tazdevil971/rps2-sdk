use crate::ffi;
use crate::sema::Sema;

use core::any::Any;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::ptr::addr_of_mut;

use alloc::boxed::Box;
use alloc::sync::Arc;

pub const DEFAULT_STACK_SIZE: u32 = 64 * 1024;
pub const MIN_STACK_SIZE: u32 = 512;
pub const MAX_STACK_SIZE: u32 = 1024 * 1024;
pub const DEFAULT_PRIORITY: u32 = 64;
pub const MIN_PRIORITY: u32 = 1;
pub const MAX_PRIORITY: u32 = 127;

pub type Result<T> = core::result::Result<T, Box<dyn Any + Send>>;

pub fn spawn<F, T>(f: F) -> ffi::Result<JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    Builder::default().spawn(f)
}

pub fn panicking() -> bool {
    rps2_panic::panicking()
}

pub fn current() -> Thread {
    Thread::current()
}

pub fn sleep() {
    unsafe {
        ffi::sleep_thread().expect("sleep_thread should never fail!");
    }
}

pub fn rotate_ready_queue(priority: u32) {
    let priority = priority.clamp(MIN_PRIORITY, MAX_PRIORITY);
    unsafe {
        ffi::rotate_thread_ready_queue(priority as _)
            .expect("rotate_thread_ready_quque should never fail!");
    }
}

// TODO: Currently bugged, required patches!
// pub unsafe fn irq_rotate_ready_queue(priority: u32) {
//     ...
// }

#[derive(Debug, Clone, Copy)]
pub struct Builder {
    stack_size: u32,
    priority: u32,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            stack_size: DEFAULT_STACK_SIZE,
            priority: DEFAULT_PRIORITY,
        }
    }

    #[allow(unused)]
    pub(crate) fn top_thread(mut self) -> Self {
        self.priority = 0;
        self
    }

    pub fn stack_size(mut self, size: u32) -> Self {
        self.stack_size = size.clamp(MIN_STACK_SIZE, MAX_STACK_SIZE);
        self
    }

    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority.clamp(MIN_PRIORITY, MAX_PRIORITY);
        self
    }

    #[must_use]
    pub fn spawn<F, T>(self, f: F) -> ffi::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        unsafe {
            // SAFETY: The signature bounds prevent any misuse of this function
            self.spawn_unchecked(f)
        }
    }

    #[must_use]
    pub unsafe fn spawn_unchecked<'a, F, T>(self, f: F) -> ffi::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'a,
        T: Send + 'a,
    {
        raw_spawn(self, f)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JoinHandle<T> {
    tid: i32,
    packet: Arc<Packet<T>>,
    stack: StackHandle,
    _marker: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn thread(&self) -> Thread {
        Thread(self.tid)
    }

    pub fn is_finished(&self) -> bool {
        unsafe {
            ffi::refer_thread_status(self.tid)
                .map(|status| (status.status & ffi::thread_status::DORMANT) != 0)
                .unwrap_or(true)
        }
    }

    pub fn join(self) -> Result<T> {
        raw_join(self)
    }

    pub fn terminate(self) {
        raw_terminate(self)
    }
}

impl<T> Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("JoinHandle");
        s.field("tid", &self.tid);
        s.finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Thread(i32);

impl Thread {
    pub fn id(&self) -> i32 {
        self.0
    }

    pub fn current() -> Self {
        Self(unsafe { ffi::get_thread_id() })
    }

    pub fn priority(&self) -> ffi::Result<u32> {
        let status = unsafe { ffi::refer_thread_status(self.0)? };
        Ok(status.current_priority as u32)
    }

    pub fn change_priority(&self, priority: u32) -> ffi::Result<()> {
        let priority = priority.clamp(MIN_PRIORITY, MAX_PRIORITY);
        unsafe { ffi::change_thread_priority(self.0, priority as _) }
    }

    pub fn suspend(&self) -> ffi::Result<()> {
        unsafe { ffi::suspend_thread(self.0) }
    }

    pub fn resume(&self) -> ffi::Result<()> {
        unsafe { ffi::resume_thread(self.0) }
    }

    pub fn wakeup(&self) -> ffi::Result<()> {
        unsafe { ffi::wakeup_thread(self.0) }
    }

    pub fn cancel_wakeup(&self) -> ffi::Result<()> {
        unsafe { ffi::cancel_wakeup_thread(self.0) }
    }

    pub unsafe fn release_wait_thread(&self) -> ffi::Result<()> {
        ffi::release_wait_thread(self.0)
    }

    // TODO: Currently bugged, required patches!
    // pub unsafe fn irq_suspend(&self) -> ffi::Result<()> {
    //     ...
    // }

    pub unsafe fn irq_resume(&self) -> ffi::Result<()> {
        ffi::irq_resume_thread(self.0)
    }

    // TODO: Currently bugged, requires patches!
    // pub unsafe fn irq_wakeup(&self) -> ffi::Result<()> {
    //     ...
    // }

    pub unsafe fn irq_cancel_wakeup(&self) -> ffi::Result<()> {
        ffi::irq_cancel_wakeup_thread(self.0)
    }

    pub unsafe fn irq_release_wait_thread(&self) -> ffi::Result<()> {
        ffi::irq_release_wait_thread(self.0)
    }
}

struct Packet<T> {
    sema: Sema,
    ret: UnsafeCell<Option<Result<T>>>,
}

unsafe impl<T: Send> Sync for Packet<T> {}

fn raw_spawn<'a, F, T>(builder: Builder, f: F) -> ffi::Result<JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    let sema = Sema::builder()
        .init_count(0)
        .max_count(1)
        .build()
        .expect("Failed to create semaphore");

    let packet = Arc::new(Packet {
        sema,
        ret: UnsafeCell::new(None),
    });

    let packet2 = Arc::clone(&packet);
    let (tid, stack) = raw_spawn2(builder, move || {
        // Catch possible unwinds
        let ret = rps2_panic::catch_unwind(f);

        unsafe {
            // SAFETY: Since we haven't signaled the semaphore yet, we are guaranteed to be the
            // only ones accessing this packet.
            *packet2.ret.get() = Some(ret);
        }

        packet2.sema.signal();
    })?;

    Ok(JoinHandle {
        tid,
        packet,
        stack,
        _marker: PhantomData,
    })
}

fn raw_join<T>(handle: JoinHandle<T>) -> Result<T> {
    // First wait for the thread to signal end of operation
    handle.packet.sema.wait();

    let ret = unsafe {
        // SAFETY: Since we waited for the semaphore, and we are the only join handle, we are
        // guaranteed to be the only ones accessing this packet.
        (*handle.packet.ret.get()).take().unwrap()
    };

    // Terminate the thread in case it's still running
    unsafe {
        let _ = ffi::terminate_thread(handle.tid);
        let _ = ffi::delete_thread(handle.tid);
    }

    // Destroy the stack
    unsafe {
        // SAFETY: After terminate/delete thread the stack is no longer in use
        handle.stack.dealloc();
    }

    ret
}

fn raw_terminate<T>(handle: JoinHandle<T>) {
    // Forcefully terminate and shut down
    unsafe {
        let _ = ffi::terminate_thread(handle.tid);
        let _ = ffi::delete_thread(handle.tid);
    }

    // Destroy the stack
    unsafe {
        // SAFETY: After terminate/delete thread the stack is no longer in use
        handle.stack.dealloc();
    }
}

fn raw_spawn2<'a, F>(builder: Builder, f: F) -> ffi::Result<(i32, StackHandle)>
where
    F: FnOnce() + Send + 'a,
{
    // Should be defined somewhere, ideally in rps2-startup
    extern "C" {
        static mut __GP: u64;
    }

    unsafe extern "C" fn launcher<'a, F>(ptr: *mut c_void)
    where
        F: FnOnce() + Send + 'a,
    {
        // Retrieve the actual closure
        let ptr = Box::from_raw(ptr as *mut F);
        // Invoke the closure
        (ptr)();
        ffi::exit_delete_thread();
    }

    let stack = unsafe {
        // SAFETY: stack_size is bound checked in the Builder
        StackHandle::alloc(builder.stack_size)
    };
    let stack_guard = scopeguard::guard((), |_| unsafe {
        stack.dealloc();
    });

    let func: unsafe extern "C" fn(*mut c_void) = launcher::<'a, F>;

    // Create the thread
    let tid = unsafe {
        ffi::create_thread(ffi::ThreadParam {
            status: 0,
            func: func as _,
            stack: stack.as_ptr() as _,
            stack_size: builder.stack_size as _,
            gp_reg: addr_of_mut!(__GP) as _,
            initial_priority: builder.priority as _,
            current_priority: 0,
            attr: 0,
            option: 0,
        })?
    };
    let tid_guard = scopeguard::guard((), |_| unsafe {
        let _ = ffi::delete_thread(tid);
    });

    // Create argument for launcher
    let args = Box::into_raw(Box::new(f));
    let args_guard = scopeguard::guard((), |_| unsafe {
        drop(Box::from_raw(args));
    });

    // Start up the thread
    unsafe {
        ffi::start_thread(tid, args as _)?;
    }

    // Finally delete all of the guards
    scopeguard::ScopeGuard::into_inner(args_guard);
    scopeguard::ScopeGuard::into_inner(tid_guard);
    scopeguard::ScopeGuard::into_inner(stack_guard);

    Ok((tid, stack))
}

#[derive(Clone, Copy)]
struct StackHandle(*mut u8, u32);

impl StackHandle {
    const ALIGN: usize = 16;

    pub unsafe fn alloc(size: u32) -> Self {
        let layout = Self::layout(size);

        let ptr = alloc::alloc::alloc(layout);
        if ptr.is_null() {
            alloc::alloc::handle_alloc_error(layout);
        }

        Self(ptr, size)
    }

    pub unsafe fn dealloc(self) {
        let layout = Self::layout(self.1);
        alloc::alloc::dealloc(self.0, layout);
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.0
    }

    pub fn layout(size: u32) -> alloc::alloc::Layout {
        alloc::alloc::Layout::from_size_align(size as _, Self::ALIGN)
            .expect("Failed to obtain stack layout")
    }
}
