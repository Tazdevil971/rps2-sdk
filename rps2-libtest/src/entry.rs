use crate::printer::TestStatus;
use crate::{printer, test, Args, Test};

use alloc::vec::Vec;

#[rps2_startup::entry]
#[cfg(not(feature = "no-entry"))]
fn testing_entry() {
    main();

    // Do not return, let the user see the result clearly
    loop {}
}

pub fn main() {
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
