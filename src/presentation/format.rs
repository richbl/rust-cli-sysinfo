use super::colors::Colors;
use crate::constants::{
    KB_PER_GB, KB_PER_MB, KB_PER_TB, LABEL_WIDTH, SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MIN,
};

/// `Threshold` controls value-based color thresholds for utility rows
///
pub enum Threshold {
    /// No threshold check; the row is rendered in the default color
    None,
    /// Apply color thresholds: yellow at `warn`, red at `crit`
    Check { value: f64, warn: f64, crit: f64 },
}

/// `color_for_threshold()` selects the appropriate ANSI color string based on the threshold
///
#[must_use]
pub fn color_for_threshold(threshold: &Threshold, c: &Colors) -> &'static str {
    match threshold {
        Threshold::Check { value, crit, .. } if value >= crit => c.red,
        Threshold::Check { value, warn, .. } if value >= warn => c.yellow,
        Threshold::Check { .. } => c.green,
        Threshold::None => c.reset,
    }
}

/// `print_row()` prints a left-aligned label/value row, coloring the value
///
pub fn print_row(label: &str, value: &str, threshold: &Threshold, c: &Colors) {
    let color = color_for_threshold(threshold, c);
    println!("{label:<LABEL_WIDTH$} {color}{value}{}", c.reset);
}

/// `print_row_error()` prints a left-aligned label/value row, coloring the value in red
///
pub fn print_row_error(label: &str, value: &str, c: &Colors) {
    println!("{label:<LABEL_WIDTH$} {}{value}{}", c.red, c.reset);
}

/// `format_uptime()` formats a duration in seconds as `DDDd:HHh:MMm:SSs`
///
#[must_use]
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / SECS_PER_DAY;
    let hours = (seconds % SECS_PER_DAY) / SECS_PER_HOUR;
    let mins = (seconds % SECS_PER_HOUR) / SECS_PER_MIN;
    let secs = seconds % SECS_PER_MIN;

    format!("{days:03}d:{hours:02}h:{mins:02}m:{secs:02}s")
}

/// `format_size()` formats a size given in kilobytes as a human-readable string with a T/G/M/K
/// suffix
///
#[must_use]
pub fn format_size(kb: u64) -> String {
    // Casting integer size constants to f64 for fractional division (precision loss possible)
    #[allow(clippy::cast_precision_loss)]
    const UNITS: &[(f64, &str)] = &[
        (KB_PER_TB as f64, "T"),
        (KB_PER_GB as f64, "G"),
        (KB_PER_MB as f64, "M"),
    ];

    #[allow(clippy::cast_precision_loss)]
    let k = kb as f64;

    // Check for any unit prefix
    for &(threshold, suffix) in UNITS {
        if k >= threshold {
            return format!("{:.1}{suffix}", k / threshold);
        }
    }

    format!("{kb}K")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `threshold_none_is_constructible()` asserts that the `Threshold::None` variant is
    /// constructible
    ///
    #[test]
    fn threshold_none_is_constructible() {
        assert!(matches!(Threshold::None, Threshold::None));
    }

    /// `threshold_check_stores_all_fields()` asserts that `Threshold::Check` correctly stores its
    /// parameters
    ///
    #[test]
    fn threshold_check_stores_all_fields() -> Result<(), String> {
        let t = Threshold::Check {
            value: 42.0,
            warn: 70.0,
            crit: 90.0,
        };

        // Unpack the variant
        let Threshold::Check { value, warn, crit } = t else {
            return Err("expected Threshold::Check, got a different variant".to_string());
        };

        assert!((value - 42.0).abs() < f64::EPSILON);
        assert!((warn - 70.0).abs() < f64::EPSILON);
        assert!((crit - 90.0).abs() < f64::EPSILON);
        Ok(())
    }

    /// `color_for_threshold_none_returns_reset()` asserts that `None` returns the reset color
    ///
    #[test]
    fn color_for_threshold_none_returns_reset() {
        let c = Colors::new(true);
        assert_eq!(color_for_threshold(&Threshold::None, &c), c.reset);
    }

    /// `color_for_threshold_below_warn_returns_green()` asserts that a value < warn returns green
    ///
    #[test]
    fn color_for_threshold_below_warn_returns_green() {
        let c = Colors::new(true);
        let t = Threshold::Check {
            value: 50.0,
            warn: 70.0,
            crit: 90.0,
        };
        assert_eq!(color_for_threshold(&t, &c), c.green);
    }

    /// `color_for_threshold_above_warn_returns_yellow()` asserts that a value >= warn returns yellow
    ///
    #[test]
    fn color_for_threshold_above_warn_returns_yellow() {
        let c = Colors::new(true);
        let t = Threshold::Check {
            value: 75.0,
            warn: 70.0,
            crit: 90.0,
        };
        assert_eq!(color_for_threshold(&t, &c), c.yellow);
    }

    /// `color_for_threshold_above_crit_returns_red()` asserts that a value >= crit returns red
    ///
    #[test]
    fn color_for_threshold_above_crit_returns_red() {
        let c = Colors::new(true);
        let t = Threshold::Check {
            value: 95.0,
            warn: 70.0,
            crit: 90.0,
        };
        assert_eq!(color_for_threshold(&t, &c), c.red);
    }
}
