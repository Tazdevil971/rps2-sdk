use core::ffi::CStr;

static mut ARGC: u32 = 0;
static mut ARGV: *const *const u8 = core::ptr::null();

pub unsafe fn setup_argcv(argc: u32, argv: *const *const u8) {
    ARGC = argc;
    ARGV = argv;
}

pub fn args() -> Args {
    unsafe {
        Args {
            idx: 0,
            argc: ARGC,
            argv: ARGV,
        }
    }
}

pub struct Args {
    idx: u32,
    argc: u32,
    argv: *const *const u8,
}

impl Args {
    unsafe fn unchecked_get(&self, i: u32) -> &'static CStr {
        let arg = self.argv.add(i as usize).read();
        CStr::from_ptr(arg as _)
    }
}

impl Iterator for Args {
    type Item = &'static CStr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.argc {
            let idx = self.idx;
            self.idx += 1;

            unsafe { Some(self.unchecked_get(idx)) }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.argc - self.idx;
        (size as _, Some(size as _))
    }
}
