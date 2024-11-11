use crate::printer::TestStatus;
use crate::{printer, test, Args, Test};

use alloc::vec::Vec;

// Bypass normal lang = "start" and directly provide a main function, this deactivates and rust main and directly connects to the startup code
#[cfg(feature = "start")]
#[no_mangle]
extern "C" fn main(_argc: u32, _argv: *const *const u8) -> i32 {
    start();

    // Do not return, let the user see the result clearly
    loop {}
}

pub fn start() {
    let args = Args::parse_args();

    if args.list {
        let tests = Test::tests().filter(|test| test.is_filtered(&args));

        printer::list_tests(&args, tests);
    } else {
        let tests = Test::tests().collect::<Vec<_>>();
        let filtered = tests
            .iter()
            .filter(|test| test.is_filtered(&args))
            .collect::<Vec<_>>();

        let mut printer = printer::run_start(&args, filtered.len(), tests.len() - filtered.len());

        for test in &filtered {
            printer.test_start(test);

            if test.is_ignored(&args) {
                printer.test_result(test, TestStatus::Ignored(test.ignore_reason()));
            } else {
                let (success, capture) = test::run(&args, test);
                if success {
                    printer.test_result(test, TestStatus::Success(capture));
                } else {
                    printer.test_result(test, TestStatus::Failed(capture));
                }
            }
        }

        printer.run_finish();
    }
}
