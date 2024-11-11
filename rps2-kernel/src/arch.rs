use core::arch::asm;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(align(64))]
pub struct CacheAligned<T>(pub T);

impl<T> Deref for CacheAligned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for CacheAligned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// The and `SYNC` instruction waits until the preceding loads or stores are completed. The
/// completion of loads indicates when the data is written into the destination register and the
/// completion of stores indicates when the data is written into the data cache or the scratch-pad
/// RAM or when the data is sent on the processor bus and the SYSDACK* signal is asserted. Also,
/// it flushes the uncached accelerated buffer and writeback buffer. In this way, load and store
/// instructions issued before `SYNC` are guaranteed to execute before load and store instructions
/// following `SYNC` are executed, in orderly sequence.
#[inline(always)]
pub unsafe fn sync() {
    asm!("sync");
}

/// The and `SYNC.L` instruction waits until the preceding loads or stores are completed. The
/// completion of loads indicates when the data is written into the destination register and the
/// completion of stores indicates when the data is written into the data cache or the scratch-pad
/// RAM or when the data is sent on the processor bus and the SYSDACK* signal is asserted. Also,
/// it flushes the uncached accelerated buffer and writeback buffer. In this way, load and store
/// instructions issued before `SYNC.L` are guaranteed to execute before load and store
/// instructions following `SYNC.L` are executed, in orderly sequence.
#[inline(always)]
pub unsafe fn syncl() {
    asm!("sync 0x00");
}

/// The `SYNC.P` instruction waits until the preceding instruction is completed with the exception
/// of multiply, divide, multicycle COP1 or COP2 operations or a pending load.
#[inline(always)]
pub unsafe fn syncp() {
    asm!("sync 0x10");
}

/// CACHE Data Cache Hit WriteBack INvalidate.
#[inline(always)]
pub unsafe fn cache_dhwbin(ptr: *const (), len: usize) {
    // Align start pointer to lane boundary
    let mut ptr = (ptr as usize) & !(64 - 1);
    // Get number of lanes to clear
    let len = (len + 63) / 64;

    // First align to a 512 byte boundary
    for _ in 0..(len & 7) {
        asm!(
            ".set noat",
            "sync",
            "cache 0x18, 0({})",
            "sync",
            in(reg) ptr
        );
        ptr += 64;
    }

    // Then do the rest in 512 bytes batches
    for _ in 0..(len >> 3) {
        asm!(
            ".set noat",
            "sync",
            "cache 0x18, 0({0})",
            "sync",
            "cache 0x18, 64({0})",
            "sync",
            "cache 0x18, 128({0})",
            "sync",
            "cache 0x18, 192({0})",
            "sync",
            "cache 0x18, 256({0})",
            "sync",
            "cache 0x18, 320({0})",
            "sync",
            "cache 0x18, 384({0})",
            "sync",
            "cache 0x18, 448({0})",
            "sync",
            in(reg) ptr
        );
        ptr += 512;
    }
}

#[inline(always)]
pub fn uncached_seg<T>(ptr: *const T) -> *const T {
    ((ptr as usize) | 0x20000000) as *const T
}

#[inline(always)]
pub fn uncached_seg_mut<T>(ptr: *mut T) -> *mut T {
    ((ptr as usize) | 0x20000000) as *mut T
}

#[inline]
pub fn is_uncached_seg<T>(ptr: *const T) -> bool {
    ((ptr as usize) & 0x20000000) != 0
}

#[inline(always)]
pub fn are_interrupts_enabled() -> bool {
    let status: u32;
    unsafe {
        asm!(
            ".set noat",
            "mfc0 {}, $12",
            out(reg) status
        );
    }

    (status & 0x10000) != 0
}

#[inline(always)]
pub fn disable_interrupts() {
    while are_interrupts_enabled() {
        unsafe {
            asm!(".set noat", "di", "sync 0x10",);
        }
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("ei");
    }
}

pub struct IntrDisableGuard(bool);

impl Drop for IntrDisableGuard {
    fn drop(&mut self) {
        if self.0 {
            crate::arch::enable_interrupts()
        }
    }
}

pub fn interrupt_disable_guard() -> IntrDisableGuard {
    let status = are_interrupts_enabled();
    if status {
        disable_interrupts();
    }

    IntrDisableGuard(status)
}

#[macro_export]
macro_rules! interrupt_disable_guard {
    () => {
        let _guard = $crate::arch::interrupt_disable_guard();
    };
}

pub mod cop0 {
    use super::*;

    pub fn get_count() -> u32 {
        let mut count;
        unsafe {
            asm!(
                ".set noat",
                "mfc0 {}, $9",
                out(reg) count
            );
        }
        count
    }
}
