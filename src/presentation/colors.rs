use crate::constants::{BOLD, CYAN, GREEN, RED, RESET, YELLOW};

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
    #[must_use]
    pub const fn new(enabled: bool) -> Self {
        if enabled {
            Self {
                red: RED,
                green: GREEN,
                yellow: YELLOW,
                cyan: CYAN,
                bold: BOLD,
                reset: RESET,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Colors::new(true) test

    #[test]
    /// `enabled_colors_are_non_empty()` asserts that ANSI color fields are non-empty when enabled
    ///
    fn enabled_colors_are_non_empty() {
        let c = Colors::new(true);
        assert!(!c.red.is_empty(), "red should be non-empty when enabled");
        assert!(
            !c.green.is_empty(),
            "green should be non-empty when enabled"
        );
        assert!(
            !c.yellow.is_empty(),
            "yellow should be non-empty when enabled"
        );
        assert!(!c.cyan.is_empty(), "cyan should be non-empty when enabled");
        assert!(!c.bold.is_empty(), "bold should be non-empty when enabled");
        assert!(
            !c.reset.is_empty(),
            "reset should be non-empty when enabled"
        );
    }

    #[test]
    /// `enabled_colors_start_with_escape_sequence()` asserts that ANSI color fields start with the
    /// escape sequence when enabled
    ///
    fn enabled_colors_start_with_escape_sequence() {
        let c = Colors::new(true);
        // All ANSI codes must begin with ESC (0x1b)
        assert!(c.red.starts_with('\x1b'), "red must start with ESC");
        assert!(c.green.starts_with('\x1b'), "green must start with ESC");
        assert!(c.yellow.starts_with('\x1b'), "yellow must start with ESC");
        assert!(c.cyan.starts_with('\x1b'), "cyan must start with ESC");
        assert!(c.bold.starts_with('\x1b'), "bold must start with ESC");
        assert!(c.reset.starts_with('\x1b'), "reset must start with ESC");
    }

    // Colors::new(false) test

    #[test]
    /// `disabled_colors_are_all_empty()` asserts that color fields are empty when disabled
    ///
    fn disabled_colors_are_all_empty() {
        let c = Colors::new(false);
        assert!(c.red.is_empty(), "red should be empty when disabled");
        assert!(c.green.is_empty(), "green should be empty when disabled");
        assert!(c.yellow.is_empty(), "yellow should be empty when disabled");
        assert!(c.cyan.is_empty(), "cyan should be empty when disabled");
        assert!(c.bold.is_empty(), "bold should be empty when disabled");
        assert!(c.reset.is_empty(), "reset should be empty when disabled");
    }

    // Threshold variants test

    #[test]
    /// `threshold_none_is_constructible()` asserts that the `Threshold::None` variant is
    /// constructible
    ///
    fn threshold_none_is_constructible() {
        assert!(matches!(Threshold::None, Threshold::None));
    }

    #[test]
    /// `threshold_check_stores_all_fields()` asserts that `Threshold::Check` correctly stores its
    /// parameters
    ///
    fn threshold_check_stores_all_fields() {
        let t = Threshold::Check {
            value: 42.0,
            warn: 70.0,
            crit: 90.0,
        };
        // Verify pattern-match access to all fields
        if let Threshold::Check { value, warn, crit } = t {
            assert!((value - 42.0).abs() < f64::EPSILON);
            assert!((warn - 70.0).abs() < f64::EPSILON);
            assert!((crit - 90.0).abs() < f64::EPSILON);
        } else {
            panic!("expected Threshold::Check");
        }
    }
}
