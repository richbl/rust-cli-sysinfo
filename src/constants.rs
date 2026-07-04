/// Application name displayed in the header
pub const APP_NAME: &str = "Rust-CLI-SysInfo";

/// Indentation prefix applied to every output row and header line
///
/// Change this single value to adjust the left-margin of all utility output.
pub const INDENT: &str = "  ";

/// Horizontal separator line used for visual sectioning
pub const SEP: &str =
    "───────────────────────────────────────────────────────────────────────────────────";

/// Label width for left-aligned labels
///
pub const LABEL_WIDTH: usize = 16;

/// Application version, sourced from Cargo package version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Clears the entire terminal screen and resets cursor to top-left
pub const CLEAR_SCREEN: &str = "\x1bc";

/// Clears the current line and returns cursor to its start
pub const CLEAR_LINE: &str = "\r\x1b[2K";

// ANSI escape sequences for terminal color output
pub const RED: &str = "\x1b[0;31m";
pub const GREEN: &str = "\x1b[0;32m";
pub const YELLOW: &str = "\x1b[0;33m";
pub const CYAN: &str = "\x1b[0;36m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

// Time conversions for `format_uptime()`
pub const SECS_PER_MIN: u64 = 60;
pub const SECS_PER_HOUR: u64 = 3_600;
pub const SECS_PER_DAY: u64 = 86_400;

// Size conversions for `format_size()`
pub const KB_PER_MB: u64 = 1_024;
pub const KB_PER_GB: u64 = 1_048_576;
pub const KB_PER_TB: u64 = 1_073_741_824;

// Warning/Critical thresholds percentages
pub const CPU_WARN_PCT: f64 = 70.0;
pub const CPU_CRIT_PCT: f64 = 90.0;
pub const MEM_WARN_PCT: f64 = 75.0;
pub const MEM_CRIT_PCT: f64 = 90.0;
pub const DISK_WARN_PCT: f64 = 80.0;
pub const DISK_CRIT_PCT: f64 = 95.0;
