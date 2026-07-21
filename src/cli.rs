use std::io::{self, IsTerminal};
use std::process;

use crate::constants::{SEP_FALLBACK, VERSION};
use crate::core::registry::ServiceRegistry;
use crate::core::utils::generate_title;
use crate::presentation::colors::Colors;
use crate::slot::{self, SlotFilter};

/// `Opts` contains the parsed command-line options for the utility
pub struct Opts {
    pub clear: bool,                       // Clear the terminal before displaying output
    pub color: bool,                       // Enable ANSI color output
    pub disk_mount: String,                // Filesystem mount path to report disk usage for
    pub cpu_sample_ms: u64,                // CPU sampling window duration in milliseconds
    pub service_key_color: Option<String>, // Custom hex color for service label keys
    pub slot_filter: SlotFilter,           // Service selection and ordering
}

/// `default_disk_mount()` returns a platform-appropriate default path for `-d/--disk`
///
fn default_disk_mount() -> String {
    if cfg!(windows) {
        "C:\\".to_string()
    } else {
        "/home".to_string()
    }
}

/// `Opts` implements the command-line options parser
impl Opts {
    /// `from_args()` parses `argv` into [`Opts`], printing usage and exiting on any error
    ///
    pub fn from_args() -> Self {
        let mut clear = true;
        let mut color = io::stdout().is_terminal();
        let mut disk_mount = default_disk_mount();
        let mut cpu_sample_ms = 250_u64;
        let mut service_key_color: Option<String> = None;
        let mut slot_filter = SlotFilter::Default;

        let mut parser = lexopt::Parser::from_env();

        // Parse command-line options
        while let Some(arg) = parser.next().unwrap_or_else(|e| fail(&format!("{e}"))) {
            match arg {
                lexopt::Arg::Short('n') | lexopt::Arg::Long("no-clear") => clear = false,
                lexopt::Arg::Short('o') | lexopt::Arg::Long("no-color") => color = false,
                lexopt::Arg::Short('d') | lexopt::Arg::Long("disk") => {
                    disk_mount = parse_disk(&mut parser);
                }
                lexopt::Arg::Short('c') | lexopt::Arg::Long("cpu-sample-rate") => {
                    cpu_sample_ms = parse_cpu_sample_ms(&mut parser);
                }
                lexopt::Arg::Short('k') | lexopt::Arg::Long("service-key-color") => {
                    service_key_color = Some(parse_service_key_color(&mut parser));
                }
                lexopt::Arg::Short('s') | lexopt::Arg::Long("services") => {
                    slot_filter = parse_slot_filter(&mut parser);
                }
                lexopt::Arg::Short('h') | lexopt::Arg::Long("help") => {
                    print_usage(&Colors::new(color, None));
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
            service_key_color,
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

/// `parse_service_key_color()` reads and validates the required hex color argument for `-k`
///
/// Accepts colors in `#RRGGBB` format (e.g., `"#ff8800"`). Both uppercase and lowercase
/// hex digits are accepted
///
fn parse_service_key_color(parser: &mut lexopt::Parser) -> String {
    let raw = parser
        .value()
        .unwrap_or_else(|_| {
            fail("-k/--service-key-color requires a hex color (e.g., -k \"#ff8800\")")
        })
        .to_string_lossy()
        .into_owned();

    validate_hex_color(&raw).unwrap_or_else(|e| fail(&e))
}

/// `validate_hex_color()` parses a `#RRGGBB` string and returns it if valid, or an error message
///
pub fn validate_hex_color(raw: &str) -> Result<String, String> {
    let bytes = raw.as_bytes();

    if bytes.first() != Some(&b'#') || bytes.len() != 7 {
        return Err(format!(
            "invalid color '{raw}': expected format #RRGGBB (e.g., #ff8800)"
        ));
    }

    // Validate that all six nibbles are valid ASCII hex digits
    for &b in &bytes[1..] {
        if !b.is_ascii_hexdigit() {
            return Err(format!(
                "invalid color '{raw}': '{}' is not a valid hex digit",
                b as char
            ));
        }
    }

    Ok(raw.to_ascii_lowercase())
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
/// Note: only *syntax* is validated here (non-empty). Whether each token names a service that
/// actually exists can only be known once `build.rs`'s discovered services are loaded — that
/// check happens in [`crate::slot::SlotFilter::resolve`], called immediately after the registry
/// is constructed in `main()`
///
fn parse_slot_list(input: &str) -> SlotFilter {
    let tokens = slot::parse_token_list(input).unwrap_or_else(|e| fail(&e));
    SlotFilter::Custom(tokens)
}

/// `fail_unknown_token()` prints an unknown-service-token error, listing the tokens actually
/// discovered by `build.rs`, and exits
///
pub fn fail_unknown_token(token: &str, registry: &ServiceRegistry) -> usize {
    let available: String = registry
        .all_meta()
        .map(|m| m.token)
        .collect::<Vec<_>>()
        .join(", ");

    fail(&format!(
        "Unknown service token '{token}' (available: {available})\n\
         (run `-s` with no argument to see available service tokens)"
    ))
}

/// `print_usage()` prints a formatted usage/help message to stdout
///
pub fn print_usage(c: &Colors) {
    let options_text = format!(
        "  Options:
    {cyan}-s, --services [TOKENS]{reset}           Select and order available services:
                                        Run -s with no arguments to see available tokens

    {cyan}-d, --disk <path>{reset}                 Disk mount to report disk usage [default: /home or C:\\]
    {cyan}-c, --cpu-sample-rate <ms>{reset}        CPU sampling window in milliseconds [default: 250]
    {cyan}-k, --service-key-color <#RRGGBB>{reset} Service label color in hex (e.g., \"#ff8800\")
    {cyan}-n, --no-clear{reset}                    Skip clearing the terminal before output
    {cyan}-o, --no-color{reset}                    Disable ANSI color output
    {cyan}-h, --help{reset}                        Show this help message and exit",
        cyan = c.cyan,
        reset = c.reset
    );

    println!(
        "\n  {}{}{}\n  {}\n  Usage: {} {}[OPTIONS]{}\n\n{}\n",
        c.bold,
        c.cyan,
        generate_title(SEP_FALLBACK.chars().count()),
        c.reset,
        env!("CARGO_PKG_NAME"),
        c.cyan,
        c.reset,
        options_text,
    );

    println!(
        "  {}{}{}\n  v{}{}",
        c.bold, c.cyan, SEP_FALLBACK, VERSION, c.reset
    );
}

/// `fail()` prints an error message with usage instructions and exits
///
fn fail(msg: &str) -> ! {
    let c = Colors::new(true, None);
    eprintln!("\n  {}{}{}", c.red, msg, c.reset);
    print_usage(&c);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `valid_lowercase_hex_is_accepted()` asserts that a correctly formatted lowercase hex color
    /// is accepted and returned in canonical lowercase form
    ///
    #[test]
    fn valid_lowercase_hex_is_accepted() {
        assert_eq!(validate_hex_color("#ff8800"), Ok("#ff8800".to_string()));
    }

    /// `valid_uppercase_hex_is_normalized_to_lowercase()` asserts that uppercase hex digits are
    /// normalized to lowercase
    ///
    #[test]
    fn valid_uppercase_hex_is_normalized_to_lowercase() {
        assert_eq!(validate_hex_color("#FF8800"), Ok("#ff8800".to_string()));
    }

    /// `valid_black_is_accepted()` asserts that the boundary value `#000000` is accepted
    ///
    #[test]
    fn valid_black_is_accepted() {
        assert!(validate_hex_color("#000000").is_ok());
    }

    /// `valid_white_is_accepted()` asserts that the boundary value `#ffffff` is accepted
    ///
    #[test]
    fn valid_white_is_accepted() {
        assert!(validate_hex_color("#ffffff").is_ok());
    }

    /// `missing_hash_prefix_is_rejected()` asserts that a color without the leading `#` is rejected
    ///
    #[test]
    fn missing_hash_prefix_is_rejected() {
        assert!(validate_hex_color("ff8800").is_err());
    }

    /// `too_short_is_rejected()` asserts that a color string shorter than 7 chars is rejected
    ///
    #[test]
    fn too_short_is_rejected() {
        assert!(validate_hex_color("#ff88").is_err());
    }

    /// `too_long_is_rejected()` asserts that a color string longer than 7 chars is rejected
    ///
    #[test]
    fn too_long_is_rejected() {
        assert!(validate_hex_color("#ff880011").is_err());
    }

    /// `invalid_hex_digit_is_rejected()` asserts that a non-hex character in the color is rejected
    ///
    #[test]
    fn invalid_hex_digit_is_rejected() {
        assert!(validate_hex_color("#gg0000").is_err());
    }

    /// `empty_string_is_rejected()` asserts that an empty string is rejected
    ///
    #[test]
    fn empty_string_is_rejected() {
        assert!(validate_hex_color("").is_err());
    }
}
