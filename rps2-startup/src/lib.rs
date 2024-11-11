#![feature(asm_experimental_arch)]
#![cfg_attr(not(feature = "no-start"), feature(lang_items))]
#![cfg_attr(not(feature = "no-start"), allow(internal_features))]
#![no_std]
use core::arch::global_asm;
use core::fmt::Debug;
use core::ptr::{addr_of, addr_of_mut};

#[repr(C)]
pub struct RawArgs {
    argc: u32,
    argv: [*const u8; 16],
    payload: [u8; 256],
}

// Used by stage0
#[no_mangle]
unsafe extern "C" fn root_func() -> ! {
    rps2_kernel::os::exit_thread();
}

#[no_mangle]
static mut ARGS: RawArgs = unsafe { core::mem::zeroed() };

global_asm!(
    r#"
# FIXME: Workaround for llvm-project#98673 and llvm-project#97685
.set arch=r5900
.text

# Default stack size for main is 64kb
.equ stack_size, (64 * 1024)

.globl __stage0_entry
.globl __stage1_entry
.globl __GP
.globl __BSS_START
.globl __BSS_END

# Stuff imported from rust side
.globl root_func
.globl ARGS

__stage0_entry:
    sync 0x10
    
    # Clear out .bss
    la $t0, __BSS_START
    la $t1, __BSS_END
clear_bss:
    sq $zero, 0($v0)
    addiu $v0, $v0, 0x10
    sltu $t2, $v0, $v1
    bne $t2, $zero, clear_bss

    # Initialize stack
    la $a0, __GP
    move $gp, $a0
    li $a1, -1
    li $a2, stack_size
    la $a3, ARGS
    la $t0, root_func
    li $v1, 0x3c
    syscall
    move $sp, $v0

    # Jump directly to stage1
    jal __stage1_entry

    # stage1 is not supposed to return
"#
);

extern "C" {
    static mut __HEAP_START: u8;

    static __preinit_array_start: u8;
    static __preinit_array_end: u8;
    static __init_array_start: u8;
    static __init_array_end: u8;
    static __fini_array_start: u8;
    static __fini_array_end: u8;
}

pub fn start_of_heap() -> *mut u8 {
    // Also destroy provenance to prevent UB
    addr_of_mut!(__HEAP_START) as usize as *mut u8
}

pub fn end_of_heap() -> *mut u8 {
    unsafe { rps2_kernel::os::end_of_heap() as _ }
}

unsafe fn invoke_array(start: *const u8, end: *const u8) {
    // Invoke all items inside of a .*_array section, given start and end
    let len = (end as usize - start as usize) / core::mem::size_of::<usize>();
    let ptr = start as *const unsafe extern "C" fn();

    let array = core::slice::from_raw_parts(ptr, len);

    for f in array {
        (*f)();
    }
}

#[no_mangle]
unsafe extern "C" fn __stage1_entry() -> ! {
    // Perform immediate jump to rust, catching any stray panics
    let res = rps2_panic::catch_unwind(|| unsafe { __stage2_entry() });

    match res {
        Ok(res) => rps2_kernel::os::exit(res),
        Err(err) => {
            // Critical error, DO NOT DROP THE PANIC PAYLOAD
            core::mem::forget(err);
            rps2_kernel::os::exit(101)
        }
    }
}

unsafe fn __stage2_entry() -> i32 {
    let argc = ARGS.argc;
    #[allow(static_mut_refs)]
    let argv = ARGS.argv.as_ptr();

    // Setup heap
    rps2_kernel::os::setup_heap(addr_of_mut!(__HEAP_START) as _, -1);

    // Initialize actual allocator if enabled
    #[cfg(feature = "alloc")]
    rps2_allocator::init(start_of_heap(), end_of_heap());

    // Enable interrupts just before rust starts
    rps2_kernel::arch::enable_interrupts();

    // Initialize env module
    #[allow(static_mut_refs)]
    rps2_kernel::env::setup_argcv(argc, argv);

    // Perform standard libc initialization
    invoke_array(
        addr_of!(__preinit_array_start) as usize as *mut u8,
        addr_of!(__preinit_array_end) as usize as *mut u8,
    );

    invoke_array(
        addr_of!(__init_array_start) as usize as *mut u8,
        addr_of!(__init_array_end) as usize as *mut u8,
    );

    // Invoke user code
    let res = main(argc, argv);

    // Perform standard libc deinitialization
    invoke_array(
        addr_of!(__fini_array_start) as usize as *mut u8,
        addr_of!(__fini_array_end) as usize as *mut u8,
    );

    res
}

extern "C" {
    fn main(argc: u32, argv: *const *const u8) -> i32;
}

#[cfg(not(feature = "no-start"))]
#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
    _sigpipe: u8,
) -> isize {
    rps2_panic::catch_unwind(|| main().report()).unwrap_or(101) as _
}

pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

impl Termination for i32 {
    fn report(self) -> i32 {
        self
    }
}

impl<T: Termination, E: Debug> Termination for Result<T, E> {
    fn report(self) -> i32 {
        match self {
            Ok(val) => val.report(),
            Err(err) => {
                rps2_kernel::kprintln!("Error: {err:?}");
                1
            }
        }
    }
}
