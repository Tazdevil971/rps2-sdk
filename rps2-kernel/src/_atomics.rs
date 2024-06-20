// Taken from https://github.com/rust-lang/compiler-builtins/blob/06db2de1c098f9c3cf69a52e7a273f19f0dce061/src/arm_linux.rs#L121

use crate::interrupt_disable_guard;

// Generic atomic read-modify-write operation
unsafe fn atomic_rmw<T, F, G>(ptr: *mut T, f: F, g: G) -> T
where
    T: Copy,
    F: Fn(T) -> T,
    G: Fn(T, T) -> T,
{
    interrupt_disable_guard!();
    let cur = *ptr;
    let new = f(cur);
    *ptr = new;
    g(cur, new)
}

// Generic atomic compare-exchange operation
unsafe fn atomic_cmpxchg<T>(ptr: *mut T, old: T, new: T) -> T
where
    T: Copy + PartialEq,
{
    interrupt_disable_guard!();
    let cur = *ptr;
    if cur == old {
        *ptr = new;
    }
    cur
}

macro_rules! atomic_rmw {
    ($name:ident, $ty:ty, $op:expr, $fetch:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr: *mut $ty, val: $ty) -> $ty {
            atomic_rmw(ptr, |x| $op(x as $ty, val), |old, new| $fetch(old, new))
        }
    };

    (@old $name:ident, $ty:ty, $op:expr) => {
        atomic_rmw!($name, $ty, $op, |old, _| old);
    };

    (@new $name:ident, $ty:ty, $op:expr) => {
        atomic_rmw!($name, $ty, $op, |_, new| new);
    };
}
macro_rules! atomic_cmpxchg {
    ($name:ident, $ty:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr: *mut $ty, oldval: $ty, newval: $ty) -> $ty {
            atomic_cmpxchg(ptr, oldval, newval)
        }
    };
}

atomic_rmw!(@old __sync_fetch_and_add_1, u8, |a: u8, b: u8| a.wrapping_add(b));
atomic_rmw!(@old __sync_fetch_and_add_2, u16, |a: u16, b: u16| a.wrapping_add(b));
atomic_rmw!(@old __sync_fetch_and_add_4, u32, |a: u32, b: u32| a.wrapping_add(b));

atomic_rmw!(@new __sync_add_and_fetch_1, u8, |a: u8, b: u8| a.wrapping_add(b));
atomic_rmw!(@new __sync_add_and_fetch_2, u16, |a: u16, b: u16| a.wrapping_add(b));
atomic_rmw!(@new __sync_add_and_fetch_4, u32, |a: u32, b: u32| a.wrapping_add(b));

atomic_rmw!(@old __sync_fetch_and_sub_1, u8, |a: u8, b: u8| a.wrapping_sub(b));
atomic_rmw!(@old __sync_fetch_and_sub_2, u16, |a: u16, b: u16| a.wrapping_sub(b));
atomic_rmw!(@old __sync_fetch_and_sub_4, u32, |a: u32, b: u32| a.wrapping_sub(b));

atomic_rmw!(@new __sync_sub_and_fetch_1, u8, |a: u8, b: u8| a.wrapping_sub(b));
atomic_rmw!(@new __sync_sub_and_fetch_2, u16, |a: u16, b: u16| a.wrapping_sub(b));
atomic_rmw!(@new __sync_sub_and_fetch_4, u32, |a: u32, b: u32| a.wrapping_sub(b));

atomic_rmw!(@old __sync_fetch_and_and_1, u8, |a: u8, b: u8| a & b);
atomic_rmw!(@old __sync_fetch_and_and_2, u16, |a: u16, b: u16| a & b);
atomic_rmw!(@old __sync_fetch_and_and_4, u32, |a: u32, b: u32| a & b);

atomic_rmw!(@new __sync_and_and_fetch_1, u8, |a: u8, b: u8| a & b);
atomic_rmw!(@new __sync_and_and_fetch_2, u16, |a: u16, b: u16| a & b);
atomic_rmw!(@new __sync_and_and_fetch_4, u32, |a: u32, b: u32| a & b);

atomic_rmw!(@old __sync_fetch_and_or_1, u8, |a: u8, b: u8| a | b);
atomic_rmw!(@old __sync_fetch_and_or_2, u16, |a: u16, b: u16| a | b);
atomic_rmw!(@old __sync_fetch_and_or_4, u32, |a: u32, b: u32| a | b);

atomic_rmw!(@new __sync_or_and_fetch_1, u8, |a: u8, b: u8| a | b);
atomic_rmw!(@new __sync_or_and_fetch_2, u16, |a: u16, b: u16| a | b);
atomic_rmw!(@new __sync_or_and_fetch_4, u32, |a: u32, b: u32| a | b);

atomic_rmw!(@old __sync_fetch_and_xor_1, u8, |a: u8, b: u8| a ^ b);
atomic_rmw!(@old __sync_fetch_and_xor_2, u16, |a: u16, b: u16| a ^ b);
atomic_rmw!(@old __sync_fetch_and_xor_4, u32, |a: u32, b: u32| a ^ b);

atomic_rmw!(@new __sync_xor_and_fetch_1, u8, |a: u8, b: u8| a ^ b);
atomic_rmw!(@new __sync_xor_and_fetch_2, u16, |a: u16, b: u16| a ^ b);
atomic_rmw!(@new __sync_xor_and_fetch_4, u32, |a: u32, b: u32| a ^ b);

atomic_rmw!(@old __sync_fetch_and_nand_1, u8, |a: u8, b: u8| !(a & b));
atomic_rmw!(@old __sync_fetch_and_nand_2, u16, |a: u16, b: u16| !(a & b));
atomic_rmw!(@old __sync_fetch_and_nand_4, u32, |a: u32, b: u32| !(a & b));

atomic_rmw!(@new __sync_nand_and_fetch_1, u8, |a: u8, b: u8| !(a & b));
atomic_rmw!(@new __sync_nand_and_fetch_2, u16, |a: u16, b: u16| !(a & b));
atomic_rmw!(@new __sync_nand_and_fetch_4, u32, |a: u32, b: u32| !(a & b));

atomic_rmw!(@old __sync_fetch_and_max_1, i8, |a: i8, b: i8| a.max(b));
atomic_rmw!(@old __sync_fetch_and_max_2, i16, |a: i16, b: i16| a.max(b));
atomic_rmw!(@old __sync_fetch_and_max_4, i32, |a: i32, b: i32| a.max(b));

atomic_rmw!(@old __sync_fetch_and_umax_1, u8, |a: u8, b: u8| a.max(b));
atomic_rmw!(@old __sync_fetch_and_umax_2, u16, |a: u16, b: u16| a.max(b));
atomic_rmw!(@old __sync_fetch_and_umax_4, u32, |a: u32, b: u32| a.max(b));

atomic_rmw!(@old __sync_fetch_and_min_1, i8, |a: i8, b: i8| a.min(b));
atomic_rmw!(@old __sync_fetch_and_min_2, i16, |a: i16, b: i16| a.min(b));
atomic_rmw!(@old __sync_fetch_and_min_4, i32, |a: i32, b: i32| a.min(b));

atomic_rmw!(@old __sync_fetch_and_umin_1, u8, |a: u8, b: u8| a.min(b));
atomic_rmw!(@old __sync_fetch_and_umin_2, u16, |a: u16, b: u16| a.min(b));
atomic_rmw!(@old __sync_fetch_and_umin_4, u32, |a: u32, b: u32| a.min(b));

atomic_rmw!(@old __sync_lock_test_and_set_1, u8, |_: u8, b: u8| b);
atomic_rmw!(@old __sync_lock_test_and_set_2, u16, |_: u16, b: u16| b);
atomic_rmw!(@old __sync_lock_test_and_set_4, u32, |_: u32, b: u32| b);

atomic_cmpxchg!(__sync_val_compare_and_swap_1, u8);
atomic_cmpxchg!(__sync_val_compare_and_swap_2, u16);
atomic_cmpxchg!(__sync_val_compare_and_swap_4, u32);

pub unsafe extern "C" fn __sync_synchronize() {
    crate::arch::sync();
}
