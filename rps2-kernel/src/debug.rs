use core::fmt::{self, Write};

#[cfg(feature = "libtest-capture")]
pub mod capture {
    use super::*;

    use alloc::string::String;
    use core::cell::RefCell;
    use critical_section::Mutex;

    static OUTPUT: Mutex<RefCell<Option<String>>> = Mutex::new(RefCell::new(None));

    pub fn start() {
        critical_section::with(|cs| {
            let mut output = OUTPUT.borrow_ref_mut(cs);
            *output = Some(String::new());
        });
    }

    pub fn take() -> Option<String> {
        critical_section::with(|cs| {
            let mut output = OUTPUT.borrow_ref_mut(cs);
            output.take()
        })
    }

    pub(super) fn try_print(args: fmt::Arguments) -> bool {
        critical_section::with(|cs| {
            let mut output = OUTPUT.borrow_ref_mut(cs);
            if let Some(output) = output.as_mut() {
                output.write_fmt(args).unwrap();
                true
            } else {
                false
            }
        })
    }
}

fn kputs(s: &str) {
    let mut buf = [0u8; 1024];
    for chunk in s.as_bytes().chunks(1023) {
        // Copy the slice into the chunk
        buf[..chunk.len()].copy_from_slice(chunk);
        buf[chunk.len()] = 0;

        unsafe {
            crate::deci2::kputs(buf[..chunk.len() + 1].as_ptr() as _);
        }
    }
}

pub fn print(args: fmt::Arguments) {
    // First try to output through the capture
    #[cfg(feature = "libtest-capture")]
    if capture::try_print(args) {
        return;
    }

    // If that fails, go through with the normal path
    struct KPutsWriter;

    impl fmt::Write for KPutsWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            kputs(s);
            Ok(())
        }
    }

    KPutsWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        $crate::debug::print(::core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => {{
        $crate::kprint!("\n");
    }};
    ($($arg:tt)*) => {{
        $crate::kprint!($($arg)*);
        $crate::kprint!("\n");
    }};
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::kprintln!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        match $val {
            val => {
                $crate::kprintln!(
                    "[{}:{}:{}] {} = {:#?}",
                    file!(), line!(), column!(), stringify!($val), &val
                );
                val
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
