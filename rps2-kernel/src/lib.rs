#![feature(asm_experimental_arch)]
#![no_std]

extern crate alloc;

pub mod arch;
pub mod debug;
pub mod deci2;
pub mod env;
pub mod os;

#[cfg(feature = "atomics")]
mod _atomics;

#[cfg(feature = "critical-section")]
mod _critical_section {
    struct EeCriticalSection;

    unsafe impl critical_section::Impl for EeCriticalSection {
        unsafe fn acquire() -> bool {
            if crate::arch::are_interrupts_enabled() {
                crate::arch::disable_interrupts();
                true
            } else {
                false
            }
        }

        unsafe fn release(state: bool) {
            if state {
                crate::arch::enable_interrupts();
            }
        }
    }

    critical_section::set_impl!(EeCriticalSection);
}
