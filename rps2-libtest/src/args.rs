use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use getargs::{Opt, Options};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunIgnored {
    Yes,
    No,
    Only,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub run_ignored: RunIgnored,
    pub exclude_should_panic: bool,
    pub run_tests: bool,
    pub run_benches: bool,
    pub list: bool,
    pub help: bool,
    pub logfile: Option<String>,
    pub nocapture: bool,
    pub terse: bool,
    pub skip: Vec<String>,
    pub exact: bool,
    pub show_output: bool,
    pub filters: Vec<String>,
}

impl Args {
    pub fn parse<'a>(opts: impl Iterator<Item = &'a str>) -> Self {
        let mut include_ignored = false;
        let mut ignored = false;
        let mut exclude_should_panic = false;
        let mut test = false;
        let mut bench = false;
        let mut list = false;
        let mut help = false;
        let mut logfile = None;
        let mut nocapture = false;
        let mut format = None;
        let mut skip = Vec::new();
        let mut quiet = false;
        let mut exact = false;
        let mut show_output = false;

        let mut opts = Options::new(opts);
        while let Some(opt) = opts.next_opt().expect("argument parsing error") {
            match opt {
                Opt::Long("include-ignored") => include_ignored = true,
                Opt::Long("ignored") => ignored = true,
                // No-op, kept here for compatibility
                Opt::Long("force-run-in-process") => {}
                Opt::Long("exclude-should-panic") => exclude_should_panic = true,
                Opt::Long("test") => test = true,
                Opt::Long("bench") => bench = true,
                Opt::Long("list") => list = true,
                Opt::Long("help") | Opt::Short('h') => help = true,
                Opt::Long("logfile") => {
                    let opt = opts.value().expect("missing --logfile argument");
                    logfile = Some(opt.to_string());
                }
                Opt::Long("nocapture") => nocapture = true,
                // No-op, kept here for compatibility
                Opt::Long("test-threads") => {
                    // Consume the argument
                    let _ = opts.value().expect("missing --test-threads argument");
                }
                Opt::Long("skip") => {
                    let opt = opts.value().expect("Missing --skip argument");
                    skip.push(opt.to_string());
                }
                Opt::Long("quiet") | Opt::Short('q') => quiet = true,
                Opt::Long("exact") => exact = true,
                // No-op, kept here for compatibility
                Opt::Long("color") => {
                    // Consume the argument
                    let _ = opts.value().expect("missing --color argument");
                }
                // No-op, kept here for compatibility
                Opt::Long("format") => {
                    // Consume the argument
                    format = Some(opts.value().expect("missing --format argument"));
                }
                Opt::Long("show-output") => show_output = true,
                // No-op, kept here for compatibility
                Opt::Short('Z') => {
                    // Consume the argument
                    let _ = opts.value().expect("missing -Z argument");
                }
                // No-op, kept here for compatibility
                Opt::Long("report-time") => {}
                // No-op, kept here for compatibility
                Opt::Long("ensure-time") => {}
                // No-op, kept here for compatibility
                Opt::Long("shuffle") => {}
                // No-op, kept here for compatibility
                Opt::Long("shuffle-seed") => {
                    // Consume the argument
                    let _ = opts.value().expect("missing --shuffle-seed argument");
                }
                opt => panic!("invalid argument: {}", opt),
            }
        }

        let filters = opts.positionals().map(String::from).collect();

        let run_ignored = match (include_ignored, ignored) {
            (true, true) => panic!("found both --include-ignored and --ignored"),
            (true, false) => RunIgnored::Yes,
            (false, true) => RunIgnored::Only,
            (false, false) => RunIgnored::No,
        };

        let terse = match format {
            None if quiet => true,
            Some("terse") => true,
            None => false,
            Some("pretty") => false,
            _ => panic!("only terse or pretty are currently supported for --format"),
        };

        let run_benches = bench;
        let run_tests = !bench || test;

        Self {
            run_ignored,
            exclude_should_panic,
            run_tests,
            run_benches,
            list,
            help,
            logfile,
            nocapture,
            terse,
            skip,
            exact,
            show_output,
            filters,
        }
    }

    pub fn parse_args() -> Self {
        let args = rps2_kernel::env::args()
            .skip(1)
            .map(|arg| arg.to_str().expect("malformed arg string"));

        Self::parse(args)
    }
}
