use crate::{Args, Test};

use alloc::string::String;
use alloc::vec::Vec;

const TERSE_MODE_MAX_COLUMN: usize = 30;

#[derive(Debug, Clone)]
pub enum TestStatus {
    Success(String),
    Failed(String),
    Ignored(Option<&'static str>),
}

pub fn list_tests<'a>(args: &Args, iter: impl Iterator<Item = &'a Test>) {
    let mut count = 0;
    for test in iter {
        rps2_kernel::kprint!("{}: test\n", test.name());
        count += 1;
    }

    if !args.terse {
        if count != 0 {
            rps2_kernel::kprint!("\n");
        }

        let noun = if count == 1 { "test" } else { "tests" };
        rps2_kernel::kprint!("{count} {noun}\n");
    }
}

pub fn run_start(args: &Args, total: usize, filtered: usize) -> RunPrinter {
    let noun = if total == 1 { "test" } else { "tests" };
    rps2_kernel::kprint!("\nrunning {total} {noun}\n");

    RunPrinter {
        terse: args.terse,
        show_output: args.show_output,
        passed: 0,
        failed: 0,
        ignored: 0,
        filtered,
        count: 0,
        total,
        column: 0,

        successes: Vec::new(),
        failures: Vec::new(),
    }
}

pub struct RunPrinter {
    terse: bool,
    show_output: bool,
    passed: usize,
    failed: usize,
    ignored: usize,
    filtered: usize,
    count: usize,
    total: usize,
    column: usize,

    successes: Vec<(&'static str, String)>,
    failures: Vec<(&'static str, String)>,
}

impl RunPrinter {
    fn write_test_name(&self, name: &str, should_panic: bool) {
        if !self.terse {
            if should_panic {
                rps2_kernel::kprint!("test {name} - should panic ... ");
            } else {
                rps2_kernel::kprint!("test {name} ... ");
            }
        }
    }

    fn write_progress(&mut self) {
        self.column = 0;
        rps2_kernel::kprint!(" {}/{}\n", self.count, self.total);
    }

    fn write_progress_if_needed(&mut self) {
        if self.column % TERSE_MODE_MAX_COLUMN == TERSE_MODE_MAX_COLUMN - 1 {
            self.write_progress();
        }
    }

    fn write_ok(&mut self) {
        self.count += 1;
        self.passed += 1;

        if self.terse {
            self.column += 1;
            rps2_kernel::kprint!(".");
            self.write_progress_if_needed();
        } else {
            rps2_kernel::kprint!("ok\n");
        }
    }

    fn write_failed(&mut self, name: &str) {
        if self.terse && self.column != 0 {
            self.write_progress();
        }

        self.count += 1;
        self.failed += 1;

        if self.terse {
            rps2_kernel::kprint!("{name} --- FAILED\n");
        } else {
            rps2_kernel::kprint!("FAILED\n");
        }
    }

    fn write_ignored(&mut self, reason: Option<&'static str>) {
        self.count += 1;
        self.ignored += 1;

        if self.terse {
            self.column += 1;
            rps2_kernel::kprint!("i");
            self.write_progress_if_needed();
        } else {
            if let Some(reason) = reason {
                rps2_kernel::kprint!("ignored, {reason}\n")
            } else {
                rps2_kernel::kprint!("ignored\n")
            }
        }
    }

    pub fn test_start(&self, test: &Test) {
        self.write_test_name(test.name(), test.should_panic());
    }

    pub fn test_result(&mut self, test: &Test, status: TestStatus) {
        match status {
            TestStatus::Success(capture) => {
                if self.show_output {
                    self.successes.push((test.name(), capture));
                }

                self.write_ok();
            }
            TestStatus::Failed(capture) => {
                self.failures.push((test.name(), capture));
                self.write_failed(test.name());
            }
            TestStatus::Ignored(reason) => {
                self.write_ignored(reason);
            }
        }
    }

    pub fn run_finish(self) {
        let success = self.failed == 0;

        if self.show_output {
            rps2_kernel::kprint!("\nsuccesses:\n");

            for (name, capture) in &self.successes {
                if !capture.is_empty() {
                    rps2_kernel::kprint!("---- {name} stdout ----\n{capture}\n");
                }
            }

            rps2_kernel::kprint!("\nsuccesses:\n");
            for (name, _) in &self.successes {
                rps2_kernel::kprint!("    {name}\n");
            }
        }

        if !success {
            rps2_kernel::kprint!("\nfailures:\n");

            for (name, capture) in &self.failures {
                if !capture.is_empty() {
                    rps2_kernel::kprint!("---- {name} stdout ----\n{capture}\n");
                }
            }

            rps2_kernel::kprint!("\nfailures:\n");
            for (name, _) in &self.failures {
                rps2_kernel::kprint!("    {name}\n");
            }
        }

        rps2_kernel::kprint!("\ntest result: ");
        if success {
            rps2_kernel::kprint!("ok")
        } else {
            rps2_kernel::kprint!("FAILED")
        }

        rps2_kernel::kprint!(
            ". {} passed; {} failed; {} ignored; {} filtered out\n\n",
            self.passed,
            self.failed,
            self.ignored,
            self.filtered
        );
    }
}
