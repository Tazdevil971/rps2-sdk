use core::arch::{asm, global_asm};
use core::ffi::{c_char, c_void};

pub mod intc_cause {
    pub const GS: i32 = 0;
    pub const SBUS: i32 = 1;
    pub const VBLANK_START: i32 = 2;
    pub const VBLANK_END: i32 = 3;
    pub const VIF0: i32 = 4;
    pub const VIF1: i32 = 5;
    pub const VU0: i32 = 6;
    pub const VU1: i32 = 7;
    pub const IPU: i32 = 8;
    pub const TIMER0: i32 = 9;
    pub const TIMER1: i32 = 10;
    pub const TIMER2: i32 = 11;
    pub const TIMER3: i32 = 12;
    pub const SFIFO: i32 = 13;
    pub const VU0_WATCHDOG: i32 = 14;
}

pub mod dmac_channel {
    pub const VIF0: i32 = 0;
    pub const VIF1: i32 = 1;
    pub const GIF: i32 = 2;
    pub const FROM_IPU: i32 = 3;
    pub const TO_IPU: i32 = 4;
    pub const SIF0: i32 = 5;
    pub const SIF1: i32 = 6;
    pub const SIF2: i32 = 7;
    pub const FROM_SPR: i32 = 8;
    pub const TO_SPR: i32 = 9;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SifDmaTransfer {
    pub src: *const c_void,
    pub dst: *mut c_void,
    pub size: i32,
    pub attr: i32,
}

pub type IntcHandler = extern "C" fn(cause: i32) -> i32;
pub type IntcHandler2 = extern "C" fn(cause: i32, arg: *mut c_void, addr: *mut c_void) -> i32;
pub type AlarmHandler = extern "C" fn(alarm_id: i32, time: u16, common: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SemaParam {
    pub count: i32,
    pub max_count: i32,
    pub init_count: i32,
    pub attr: u32,
    pub option: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThreadParam {
    pub status: i32,
    pub func: *const c_void,
    pub stack: *mut c_void,
    pub stack_size: i32,
    pub gp_reg: *mut c_void,
    pub initial_priority: i32,
    pub current_priority: i32,
    pub attr: u32,
    pub option: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThreadStatus {
    pub status: i32,
    pub func: *const c_void,
    pub stack: *mut c_void,
    pub stack_size: i32,
    pub gp_reg: *mut c_void,
    pub initial_priority: i32,
    pub current_priority: i32,
    pub attr: u32,
    pub option: u32,
    pub wait_type: u32,
    pub wait_id: u32,
    pub wakeup_count: u32,
}

pub mod thread_status {
    pub const RUN: i32 = 1;
    pub const READY: i32 = 2;
    pub const WAIT: i32 = 4;
    pub const SUSPEND: i32 = 8;
    pub const WAITSUSPEND: i32 = 16;
    pub const DORMANT: i32 = 32;
}

pub mod thread_wait {
    pub const NONE: i32 = 0;
    pub const SLEEP: i32 = 1;
    pub const SEMA: i32 = 2;
}

pub mod flush_cache_op {
    pub const FLUSH_DATA: i32 = 0;
    pub const INVALIDATE_DATA: i32 = 1;
    pub const INVALIDATE_INSTRUCTION: i32 = 2;
    pub const FLUSH_BOTH: i32 = 3;
}

macro_rules! define_syscall {
    ($(#[$attr:meta])* $vis:vis fn $name:ident (
        $($aname:ident : $atype:ty),*
    ) $(-> $rtype:ty)? as $num:literal; $($rest:tt)*) => {

        global_asm!(
            concat!(".globl ", stringify!($name)),
            concat!(stringify!($name), ":"),
            concat!("li $v1, ", $num),
            "syscall",
            "jr $ra"
        );

        extern "C" {
            $(#[$attr])*
            $vis fn $name( $($aname : $atype),* ) $(-> $rtype)?;
        }

        define_syscall!($($rest)*);
    };
    () => {}
}

define_syscall! {
    pub fn reset_ee(flags: u32) as 0x01;
    pub fn set_gs_crt(interlaced: i16, pal_ntsc: i16, frame: i16) as 0x02;
    pub fn exit(status: i32) -> ! as 0x04;
    pub fn load_exec_ps2(
        filename: *const c_char,
        argc: i32,
        argv: *mut *mut c_char
    ) -> ! as 0x06;
    pub fn exec_ps2(
        entry: *mut c_void,
        gp: *mut c_void,
        argc: i32,
        argv: *mut *mut c_char
    ) -> i32 as 0x07;

    pub fn add_intc_handler(cause: i32, handler: IntcHandler, next: i32) -> i32 as 0x10;
    pub fn add_intc_handler2(
        cause: i32,
        handler: IntcHandler2,
        next: i32,
        arg: *mut c_void
    ) -> i32 as 0x10;
    pub fn remove_intc_handler(cause: i32, id: i32) -> i32 as 0x11;

    pub fn add_dmac_handler(channel: i32, handler: IntcHandler, next: i32) -> i32 as 0x12;
    pub fn add_dmac_handler2(
        channel: i32,
        handler: IntcHandler2,
        next: i32,
        arg: *mut c_void
    ) -> i32 as 0x12;
    pub fn remove_dmac_handler(channel: i32, id: i32) -> i32 as 0x13;

    pub fn _enable_intc(cause: i32) -> i32 as 0x14;
    pub fn _disable_intc(cause: i32) -> i32 as 0x15;
    pub fn _enable_dmac(channel: i32) -> i32 as 0x16;
    pub fn _disable_dmac(channel: i32) -> i32 as 0x17;

    pub fn _i_enable_intc(cause: i32) -> i32 as -0x1a;
    pub fn _i_disable_intc(cause: i32) -> i32 as -0x1b;
    pub fn _i_enable_dmac(channel: i32) -> i32 as -0x1c;
    pub fn _i_disable_dmac(channel: i32) -> i32 as -0x1d;

    pub fn create_thread(params: *mut ThreadParam) -> i32 as 0x20;
    pub fn delete_thread(tid: i32) -> i32 as 0x21;
    pub fn start_thread(tid: i32, args: *mut c_void) -> i32 as 0x22;
    pub fn exit_thread() -> ! as 0x23;
    pub fn exit_delete_thread() -> ! as 0x24;
    pub fn terminate_thread(tid: i32) -> i32 as 0x25;
    pub fn i_terminate_thread(tid: i32) -> i32 as -0x26;
    // pub fn disable_dispatch_thread() as 0x27; // Not supported
    // pub fn enable_dispatch_thread() as 0x28; // Not supported
    pub fn change_thread_priority(tid: i32, priority: i32) -> i32 as 0x29;
    pub fn i_change_thread_priority(tid: i32, priority: i32) -> i32 as -0x2a;
    pub fn rotate_thread_ready_queue(priority: i32) -> i32 as 0x2b;
    pub fn _i_rotate_thread_ready_queue(priority: i32) -> i32 as -0x2c;
    pub fn release_wait_thread(tid: i32) -> i32 as 0x2d;
    pub fn i_release_wait_thread(tid: i32) -> i32 as -0x2e;
    pub fn get_thread_id() -> i32 as 0x2f;
    pub fn _i_get_thread_id() -> i32 as -0x2f;
    pub fn refer_thread_status(tid: i32, info: *mut ThreadStatus) -> i32 as 0x30;
    pub fn i_refer_thread_status(tid: i32, info: *mut ThreadStatus) -> i32 as -0x31;
    pub fn sleep_thread() -> i32 as 0x32;
    pub fn wakeup_thread(tid: i32) -> i32 as 0x33;
    pub fn _i_wakeup_thread(tid: i32) -> i32 as -0x34;
    pub fn cancel_wakeup_thread(tid: i32) -> i32 as 0x35;
    pub fn i_cancel_wakeup_thread(tid: i32) -> i32 as -0x36;
    pub fn suspend_thread(tid: i32) -> i32 as 0x37;
    pub fn _i_suspend_thread(tid: i32) -> i32 as -0x38;
    pub fn resume_thread(tid: i32) -> i32 as 0x39;
    pub fn i_resume_thread(tid: i32) -> i32 as -0x3a;

    pub fn setup_thread(
        gp: *mut c_void,
        stack: *mut c_void,
        stack_size: i32,
        args: *mut c_void,
        root_func: *mut c_void
    ) -> *mut c_void as 0x3c;
    pub fn setup_heap(heap_start: *mut c_void, heap_size: i32) as 0x3d;
    pub fn end_of_heap() -> *mut c_void as 0x3e;

    pub fn create_sema(params: *mut SemaParam) -> i32 as 0x40;
    pub fn delete_sema(sid: i32) -> i32 as 0x41;
    pub fn signal_sema(sid: i32) -> i32 as 0x42;
    pub fn i_signal_sema(sid: i32) -> i32 as -0x43;
    pub fn wait_sema(sid: i32) -> i32 as 0x44;
    pub fn poll_sema(sid: i32) -> i32 as 0x45;
    pub fn i_poll_sema(sid: i32) -> i32 as -0x46;
    pub fn refer_sema_status(sid: i32, status: *mut SemaParam) -> i32 as 0x47;
    pub fn i_refer_sema_status(sid: i32, status: *mut SemaParam) -> i32 as -0x48;
    pub fn i_delete_sema(sid: i32) -> i32 as -0x49;

    pub fn flush_cache(op: i32) as 0x64;
    pub fn i_flush_cache(op: i32) as -0x68;

    pub fn gs_get_imr() -> u64 as 0x70;
    pub fn gs_put_imr(imr: u64) as 0x71;

    pub fn sif_dma_stat(id: u32) -> i32 as 0x76;
    pub fn i_sif_dma_stat(id: u32) -> i32 as -0x76;
    pub fn sif_set_dma(sdd: *const SifDmaTransfer, len: i32) -> u32 as 0x77;
    pub fn i_sif_set_dma(sdd: *const SifDmaTransfer, len: i32) -> u32 as -0x77;
    pub fn sif_set_d_chain() as 0x78;
    pub fn i_sif_set_d_chain() as -0x78;

    pub fn sif_set_reg(num: u32, value: i32) -> i32 as 0x79;
    pub fn sif_get_reg(num: u32) -> i32 as 0x7a;

    pub fn deci2_call(call: i32, addr: *mut u32) -> i32 as 0x7c;

    pub fn ps_mode() as 0x7d;
    pub fn machine_type() -> i32 as 0x7e;
    pub fn get_memory_size() -> i32 as 0x7f;
}

pub unsafe fn disable_intc(cause: i32) -> i32 {
    crate::interrupt_disable_guard!();

    let res = _disable_intc(cause);
    crate::arch::sync();
    res
}

pub unsafe fn enable_intc(cause: i32) -> i32 {
    crate::interrupt_disable_guard!();

    let res = _enable_intc(cause);
    crate::arch::sync();
    res
}

pub unsafe fn disable_dmac(channel: i32) -> i32 {
    crate::interrupt_disable_guard!();

    let res = _disable_dmac(channel);
    crate::arch::sync();
    res
}

pub unsafe fn enable_dmac(channel: i32) -> i32 {
    crate::interrupt_disable_guard!();

    let res = _enable_dmac(channel);
    crate::arch::sync();
    res
}

pub unsafe fn i_disable_intc(cause: i32) -> i32 {
    let res = _i_disable_intc(cause);
    crate::arch::sync();
    res
}

pub unsafe fn i_enable_intc(cause: i32) -> i32 {
    let res = _i_enable_intc(cause);
    crate::arch::sync();
    res
}

pub unsafe fn i_disable_dmac(channel: i32) -> i32 {
    let res = _i_disable_dmac(channel);
    crate::arch::sync();
    res
}

pub unsafe fn i_enable_dmac(channel: i32) -> i32 {
    let res = _i_enable_dmac(channel);
    crate::arch::sync();
    res
}

pub unsafe fn exit_handler() {
    unsafe {
        asm!("sync 0x00", "ei");
    }
}
