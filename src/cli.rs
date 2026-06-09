use std::io::{self, IsTerminal};
use std::process;

use crate::{APP_NAME, VERSION};

/// Parsed command-line options for the dashboard
pub struct Opts {
    pub clear: bool,        // Clear the terminal before displaying output
    pub color: bool,        // Enable ANSI color output
    pub disk_mount: String, // Filesystem mount path to report disk usage for
    pub cpu_sample_ms: u64, // CPU sampling window duration in milliseconds
}

/// Command-line option help text
const USAGE: &str = "
Options:
  -d, --disk <path>           Disk mount to report disk usage for [default: /home]
  -c, --cpu-sample-rate <ms>  CPU sampling window in milliseconds [default: 250]
  -n, --no-clear              Skip clearing the terminal before output
  -o, --no-color              Disable ANSI color output
  -h, --help                  Show this help message and exit";

/// Command-line options parser
impl Opts {
    /// `from_args()` parses `argv` into [`Opts`], printing usage and exiting on any error
    ///
    pub fn from_args() -> Self {
        let mut clear = true;
        let mut color = io::stdout().is_terminal();
        let mut disk_mount = "/home".to_string();
        let mut cpu_sample_ms = 250;

        // Parse command-line options
        let mut parser = lexopt::Parser::from_env();

        while let Some(arg) = parser
            .next()
            .unwrap_or_else(|e| fail(&format!("Error: {e}")))
        {
            match arg {
                lexopt::Arg::Short('n') | lexopt::Arg::Long("no-clear") => clear = false,
                lexopt::Arg::Short('o') | lexopt::Arg::Long("no-color") => color = false,

                lexopt::Arg::Short('d') | lexopt::Arg::Long("disk") => {
                    disk_mount = parser
                        .value()
                        .unwrap_or_else(|_| {
                            fail("-d/--disk requires a path argument (e.g., --disk /var)")
                        })
                        .to_string_lossy()
                        .into_owned();
                }

                lexopt::Arg::Short('c') | lexopt::Arg::Long("cpu-sample-rate") => {
                    cpu_sample_ms = parser
                        .value()
                        .ok()
                        .and_then(|v| v.to_str()?.parse().ok())
                        .filter(|&ms| ms > 0)
                        .unwrap_or_else(|| {
                            fail("-c/--cpu-sample-rate requires a positive integer (e.g., -c 250)")
                        });
                }

                lexopt::Arg::Short('h') | lexopt::Arg::Long("help") => {
                    print_usage();
                    process::exit(0);
                }

                unknown => fail(&format!("Unknown option '{}'", format_arg(unknown))),
            }
        }
        Self {
            clear,
            color,
            disk_mount,
            cpu_sample_ms,
        }
    }
}

/// `print_usage()` prints a formatted usage/help message to stdout
///
fn print_usage() {
    println!(
        "\n{} v{}\nUsage: {} [OPTIONS]\n\n{}",
        APP_NAME,
        VERSION,
        env!("CARGO_PKG_NAME"),
        USAGE.trim_start_matches('\n'),
    );
}

/// ` fail()` prints an error message with usage instructions and exits
///
fn fail(msg: &str) -> ! {
    eprintln!("\n{msg}");
    print_usage();
    process::exit(1);
}

/// `format_arg()` formats a `lexopt::Arg` into a human-readable string for error reporting
///
fn format_arg(arg: lexopt::Arg) -> String {
    match arg {
        lexopt::Arg::Long(s) => format!("--{s}"),
        lexopt::Arg::Short(c) => format!("-{c}"),
        lexopt::Arg::Value(v) => v.to_string_lossy().into_owned(),
    }
}
