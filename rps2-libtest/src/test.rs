use crate::args::{Args, RunIgnored};

use alloc::string::String;

#[derive(Debug, Clone, Copy)]
struct Ignore {
    reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct Test {
    name: &'static str,
    func: fn(),
    ignore: Option<Ignore>,
    should_panic: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TestBuilder(Test);

impl TestBuilder {
    pub const fn new(name: &'static str, func: fn()) -> Self {
        Self(Test {
            name,
            func,
            ignore: None,
            should_panic: false,
        })
    }

    pub const fn ignore(mut self) -> Self {
        self.0.ignore = Some(Ignore { reason: None });
        self
    }

    pub const fn ignore_reason(mut self, reason: &'static str) -> Self {
        self.0.ignore = Some(Ignore {
            reason: Some(reason),
        });
        self
    }

    pub const fn with_should_panic(mut self) -> Self {
        self.0.should_panic = true;
        self
    }

    pub const fn build(self) -> Test {
        self.0
    }
}

inventory::collect!(Test);

impl Test {
    pub fn tests() -> impl Iterator<Item = &'static Test> {
        inventory::iter::<Test>.into_iter()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn should_panic(&self) -> bool {
        self.should_panic
    }

    pub fn ignore(&self) -> bool {
        self.ignore.is_some()
    }

    pub fn ignore_reason(&self) -> Option<&'static str> {
        self.ignore.map(|ignore| ignore.reason).flatten()
    }

    pub fn is_filtered(&self, args: &Args) -> bool {
        let matches_filter = |filter: &String| {
            if args.exact {
                self.name == filter
            } else {
                self.name.contains(filter)
            }
        };

        // Discard all tests that don't match any of the filter
        if !args.filters.iter().any(matches_filter) && !args.filters.is_empty() {
            return false;
        }

        // Discard tests that match any of the skips
        if args.skip.iter().any(matches_filter) {
            return false;
        }

        // Discard #[should_panic] tests
        if args.exclude_should_panic && self.should_panic() {
            return false;
        }

        // Discard not ignored tests
        if args.run_ignored == RunIgnored::Only && !self.ignore() {
            return false;
        }

        true
    }

    pub fn is_ignored(&self, args: &Args) -> bool {
        args.run_ignored == RunIgnored::No && self.ignore()
    }
}

pub fn run(args: &Args, test: &Test) -> (bool, String) {
    // TODO: Due to the way this function is written it cannot be called in parallel! Maybe enforce it?

    if !args.nocapture {
        // First setup output capture
        rps2_kernel::debug::capture::start();
    }

    // Run the test
    let res = rps2_panic::catch_unwind(test.func);

    let mut output = if !args.nocapture {
        rps2_kernel::debug::capture::take().unwrap()
    } else {
        String::new()
    };

    let success = if test.should_panic() {
        res.is_err()
    } else {
        res.is_ok()
    };

    // Add a dummy line to signal the user that the test did not panic
    if !success && test.should_panic() {
        output.push_str("note: test did not panic as expected\n");
    }

    (success, output)
}
