use std::io::{self, IsTerminal};
use std::process;

use crate::presentation::colors::Colors;
use crate::slot::{ServiceSlot, SlotFilter};
use crate::{APP_NAME, SEP, VERSION};

/// `Opts` contains the parsed command-line options for the utility
pub struct Opts {
    pub clear: bool,             // Clear the terminal before displaying output
    pub color: bool,             // Enable ANSI color output
    pub disk_mount: String,      // Filesystem mount path to report disk usage for
    pub cpu_sample_ms: u64,      // CPU sampling window duration in milliseconds
    pub slot_filter: SlotFilter, // Service selection and ordering
}

/// `Opts` implements the command-line options parser
impl Opts {
    /// `from_args()` parses `argv` into [`Opts`], printing usage and exiting on any error
    ///
    pub fn from_args() -> Self {
        let mut clear = true;
        let mut color = io::stdout().is_terminal();
        let mut disk_mount = "/home".to_string();
        let mut cpu_sample_ms = 250_u64;
        let mut slot_filter = SlotFilter::Default;

        let mut parser = lexopt::Parser::from_env();

        // Parse command-line options
        while let Some(arg) = parser
            .next()
            .unwrap_or_else(|e| fail(&format!("Error: {e}")))
        {
            match arg {
                lexopt::Arg::Short('n') | lexopt::Arg::Long("no-clear") => clear = false,
                lexopt::Arg::Short('o') | lexopt::Arg::Long("no-color") => color = false,
                lexopt::Arg::Short('d') | lexopt::Arg::Long("disk") => {
                    disk_mount = parse_disk(&mut parser);
                }
                lexopt::Arg::Short('c') | lexopt::Arg::Long("cpu-sample-rate") => {
                    cpu_sample_ms = parse_cpu_sample_ms(&mut parser);
                }
                lexopt::Arg::Short('s') | lexopt::Arg::Long("services") => {
                    slot_filter = parse_slot_filter(&mut parser);
                }
                lexopt::Arg::Short('h') | lexopt::Arg::Long("help") => {
                    print_usage(&Colors::new(color));
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
            slot_filter,
        }
    }
}

/// `parse_disk()` reads and returns the required path argument for `-d/--disk`
///
fn parse_disk(parser: &mut lexopt::Parser) -> String {
    parser
        .value()
        .unwrap_or_else(|_| fail("-d/--disk requires a path argument (e.g., --disk /var)"))
        .to_string_lossy()
        .into_owned()
}

/// `parse_cpu_sample_ms()` reads and validates the required millisecond argument for `-c`
///
fn parse_cpu_sample_ms(parser: &mut lexopt::Parser) -> u64 {
    parser
        .value()
        .ok()
        .and_then(|v| v.to_str()?.parse().ok())
        .filter(|&ms: &u64| ms > 0)
        .unwrap_or_else(|| fail("-c/--cpu-sample-rate requires a positive integer (e.g., -c 250)"))
}

/// `parse_slot_filter()` reads the optional token list argument for `-s/--services`
///
fn parse_slot_filter(parser: &mut lexopt::Parser) -> SlotFilter {
    if let Some(val) = parser.optional_value() {
        return parse_slot_list(&val.to_string_lossy());
    }

    match parser.value() {
        Ok(val) => {
            let s = val.to_string_lossy();
            if s.starts_with('-') {
                fail(&format!(
                    "expected a service token list after -s, got '{s}'\n\
                     (run -s with no arguments to see available tokens)"
                ));
            }
            parse_slot_list(&s)
        }
        // No further tokens: bare `-s` requests the labeled reference table
        Err(_) => SlotFilter::ShowLabeled,
    }
}

/// `parse_slot_list()` parses a hyphen-separated token string into a `SlotFilter::Custom`
///
fn parse_slot_list(input: &str) -> SlotFilter {
    let slots = ServiceSlot::parse_list(input).unwrap_or_else(|e| fail(&format!("Error: {e}")));
    SlotFilter::Custom(slots)
}

/// `print_usage()` prints a formatted usage/help message to stdout
///
fn print_usage(c: &Colors) {
    let options_text = format!(
        "  Options:
    {cyan}-s, --services [TOKENS]{reset}     Select and order available services: 
                                Run -s with no arguments to see available tokens

    {cyan}-d, --disk <path>{reset}           Disk mount to report disk usage [default: /home]
    {cyan}-c, --cpu-sample-rate <ms>{reset}  CPU sampling window in milliseconds [default: 250]
    {cyan}-n, --no-clear{reset}              Skip clearing the terminal before output
    {cyan}-o, --no-color{reset}              Disable ANSI color output
    {cyan}-h, --help{reset}                  Show this help message and exit",
        cyan = c.cyan,
        reset = c.reset
    );

    println!(
        "\n  {}{}{}\n  {}{}\n  Usage: {} {}[OPTIONS]{}\n\n{}",
        c.bold,
        c.cyan,
        APP_NAME,
        SEP,
        c.reset,
        env!("CARGO_PKG_NAME"),
        c.cyan,
        c.reset,
        options_text,
    );

    println!("  {}{}{}\n  v{}{}", c.bold, c.cyan, SEP, VERSION, c.reset);
}

/// `fail()` prints an error message with usage instructions and exits
///
fn fail(msg: &str) -> ! {
    eprintln!("\n{msg}");
    print_usage(&Colors::new(true));
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
