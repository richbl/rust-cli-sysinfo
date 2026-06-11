/// ANSI escape sequences for terminal color output
///
pub struct Colors {
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub cyan: &'static str,
    pub bold: &'static str,

    /// Resets all attributes back to the terminal default
    pub reset: &'static str,
}

/// `Colors` constructor
impl Colors {
    /// `new` creates a `Colors` instance with ANSI codes when `enabled` is `true`, or empty
    /// strings otherwise
    ///
    pub const fn new(enabled: bool) -> Self {
        if enabled {
            Self {
                red: "\x1b[0;31m",
                green: "\x1b[0;32m",
                yellow: "\x1b[0;33m",
                cyan: "\x1b[0;36m",
                bold: "\x1b[1m",
                reset: "\x1b[0m",
            }
        } else {
            Self {
                red: "",
                green: "",
                yellow: "",
                cyan: "",
                bold: "",
                reset: "",
            }
        }
    }
}

/// `Threshold` controls value-based color thresholds for utility rows
pub enum Threshold {
    None, // No threshold check; the row is rendered in the default color
    Check { value: f64, warn: f64, crit: f64 }, // Apply thresholds: yellow at `warn`, red at `crit`
}
