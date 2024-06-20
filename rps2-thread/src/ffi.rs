#![allow(dead_code)]

use rps2_kernel::os;

use core::ffi::c_void;
use core::mem::MaybeUninit;

pub use os::{thread_status, SemaParam, ThreadParam, ThreadStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    CreateThread,
    DeleteThread,
    StartThread,
    TerminateThread,
    ChangeThreadPriority,
    RotateThreadReadyQueue,
    ReleaseWaitThread,
    ReferThreadStatus,
    SleepThread,
    WakeupThread,
    CancelWakeupThread,
    SuspendThread,
    ResumeThread,
    CreateSema,
    DeleteSema,
    SignalSema,
    WaitSema,
    PollSema,
    ReferSemaStatus,
}

use Syscall::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub syscall: Syscall,
    pub code: i32,
}

impl Error {
    pub fn new(syscall: Syscall, code: i32) -> Self {
        Self { syscall, code }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

fn handle_res(res: i32, syscall: Syscall) -> Result<i32> {
    if res < 0 {
        Err(Error::new(syscall, res))
    } else {
        Ok(res)
    }
}

fn handle_res_none(res: i32, syscall: Syscall) -> Result<()> {
    if res < 0 {
        Err(Error::new(syscall, res))
    } else {
        Ok(())
    }
}

pub unsafe fn create_thread(mut params: ThreadParam) -> Result<i32> {
    handle_res(os::create_thread(&mut params), CreateThread)
}

pub unsafe fn delete_thread(tid: i32) -> Result<()> {
    handle_res_none(os::delete_thread(tid), DeleteThread)
}

pub unsafe fn start_thread(tid: i32, args: *mut c_void) -> Result<()> {
    handle_res_none(os::start_thread(tid, args), StartThread)
}

pub unsafe fn exit_thread() -> ! {
    os::exit_thread()
}

pub unsafe fn exit_delete_thread() -> ! {
    os::exit_delete_thread()
}

pub unsafe fn terminate_thread(tid: i32) -> Result<()> {
    handle_res_none(os::terminate_thread(tid), TerminateThread)
}

pub unsafe fn irq_terminate_thread(tid: i32) -> Result<()> {
    handle_res_none(os::i_terminate_thread(tid), TerminateThread)
}

pub unsafe fn change_thread_priority(tid: i32, priority: i32) -> Result<()> {
    handle_res_none(
        os::change_thread_priority(tid, priority),
        ChangeThreadPriority,
    )
}

pub unsafe fn irq_change_thread_priority(tid: i32, priority: i32) -> Result<()> {
    handle_res_none(
        os::i_change_thread_priority(tid, priority),
        ChangeThreadPriority,
    )
}

pub unsafe fn rotate_thread_ready_queue(priority: i32) -> Result<()> {
    handle_res_none(os::rotate_thread_ready_queue(priority), TerminateThread)
}

// TODO:
// pub unsafe fn irq_rotate_thread_ready_queue(tid: i32) -> Result<()> {
//     handle_res_none(os::_i_rotate_thread_ready_queue(tid), TerminateThread)
// }

pub unsafe fn release_wait_thread(tid: i32) -> Result<()> {
    handle_res_none(os::release_wait_thread(tid), ReleaseWaitThread)
}

pub unsafe fn irq_release_wait_thread(tid: i32) -> Result<()> {
    handle_res_none(os::i_release_wait_thread(tid), ReleaseWaitThread)
}

pub unsafe fn get_thread_id() -> i32 {
    os::get_thread_id()
}

pub unsafe fn refer_thread_status(tid: i32) -> Result<ThreadStatus> {
    let mut status = MaybeUninit::uninit();
    handle_res_none(
        os::refer_thread_status(tid, status.as_mut_ptr()),
        ReferThreadStatus,
    )
    .map(|_| unsafe { status.assume_init() })
}

pub unsafe fn irq_refer_thread_status(tid: i32) -> Result<ThreadStatus> {
    let mut status = MaybeUninit::uninit();
    handle_res_none(
        os::i_refer_thread_status(tid, status.as_mut_ptr()),
        ReferThreadStatus,
    )
    .map(|_| unsafe { status.assume_init() })
}

pub unsafe fn sleep_thread() -> Result<()> {
    handle_res_none(os::sleep_thread(), SleepThread)
}

pub unsafe fn wakeup_thread(tid: i32) -> Result<()> {
    handle_res_none(os::wakeup_thread(tid), WakeupThread)
}

// TODO:
// pub unsafe fn irq_wakeup_thread(tid: i32) -> Result<()> {
//     handle_res_none(os::_i_wakeup_thread(tid), WakeupThread)
// }

pub unsafe fn cancel_wakeup_thread(tid: i32) -> Result<()> {
    handle_res_none(os::cancel_wakeup_thread(tid), CancelWakeupThread)
}

pub unsafe fn irq_cancel_wakeup_thread(tid: i32) -> Result<()> {
    handle_res_none(os::i_cancel_wakeup_thread(tid), CancelWakeupThread)
}

pub unsafe fn suspend_thread(tid: i32) -> Result<()> {
    handle_res_none(os::suspend_thread(tid), SuspendThread)
}

// TODO:
// pub unsafe fn irq_suspend_thread(tid: i32) -> Result<()> {
//     handle_res_none(os::_i_suspend_thread(tid), SuspendThread)
// }

pub unsafe fn resume_thread(tid: i32) -> Result<()> {
    handle_res_none(os::resume_thread(tid), ResumeThread)
}

pub unsafe fn irq_resume_thread(tid: i32) -> Result<()> {
    handle_res_none(os::i_resume_thread(tid), ResumeThread)
}

pub unsafe fn create_sema(mut params: SemaParam) -> Result<i32> {
    handle_res(os::create_sema(&mut params), CreateSema)
}

pub unsafe fn delete_sema(sid: i32) -> Result<()> {
    handle_res_none(os::delete_sema(sid), DeleteSema)
}

pub unsafe fn irq_delete_sema(sid: i32) -> Result<()> {
    handle_res_none(os::i_delete_sema(sid), DeleteSema)
}

pub unsafe fn signal_sema(sid: i32) -> Result<()> {
    handle_res_none(os::signal_sema(sid), SignalSema)
}

pub unsafe fn irq_signal_sema(sid: i32) -> Result<()> {
    handle_res_none(os::i_signal_sema(sid), SignalSema)
}

pub unsafe fn wait_sema(sid: i32) -> Result<()> {
    handle_res_none(os::wait_sema(sid), WaitSema)
}

pub unsafe fn poll_sema(sid: i32) -> Result<()> {
    handle_res_none(os::poll_sema(sid), PollSema)
}

pub unsafe fn irq_poll_sema(sid: i32) -> Result<()> {
    handle_res_none(os::i_poll_sema(sid), PollSema)
}

pub unsafe fn refer_sema_status(sid: i32) -> Result<SemaParam> {
    let mut status = MaybeUninit::uninit();
    handle_res_none(
        os::refer_sema_status(sid, status.as_mut_ptr()),
        ReferSemaStatus,
    )
    .map(|_| unsafe { status.assume_init() })
}

pub unsafe fn irq_refer_sema_status(sid: i32) -> Result<SemaParam> {
    let mut status = MaybeUninit::uninit();
    handle_res_none(
        os::i_refer_sema_status(sid, status.as_mut_ptr()),
        ReferSemaStatus,
    )
    .map(|_| unsafe { status.assume_init() })
}
