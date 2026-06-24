use super::colors::Colors;
use crate::constants::{
    KB_PER_GB, KB_PER_MB, KB_PER_TB, SECS_PER_DAY, SECS_PER_HOUR, SECS_PER_MIN,
};

/// `Threshold` controls value-based color thresholds for utility rows
pub enum Threshold {
    None, // No threshold check; the row is rendered in the default color
    Check { value: f64, warn: f64, crit: f64 }, // Apply thresholds: yellow at `warn`, red at `crit`
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
    println!("{label:<16} {color}{value}{}", c.reset);
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
    // Divisors are powers of 1024 (all expressed in KB)

    #[allow(clippy::cast_precision_loss)]
    let k = kb as f64;

    if k >= KB_PER_TB {
        format!("{:.1}T", k / KB_PER_TB)
    } else if k >= KB_PER_GB {
        format!("{:.1}G", k / KB_PER_GB)
    } else if k >= KB_PER_MB {
        format!("{:.1}M", k / KB_PER_MB)
    } else {
        format!("{kb}K")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    /// `color_for_threshold_none_returns_reset()` asserts that `None` returns the reset color
    ///
    fn color_for_threshold_none_returns_reset() {
        let c = Colors::new(true);
        assert_eq!(color_for_threshold(&Threshold::None, &c), c.reset);
    }

    #[test]
    /// `color_for_threshold_below_warn_returns_green()` asserts that a value < warn returns green
    ///
    fn color_for_threshold_below_warn_returns_green() {
        let c = Colors::new(true);
        let t = Threshold::Check {
            value: 50.0,
            warn: 70.0,
            crit: 90.0,
        };
        assert_eq!(color_for_threshold(&t, &c), c.green);
    }

    #[test]
    /// `color_for_threshold_above_warn_returns_yellow()` asserts that a value >= warn returns yellow
    ///
    fn color_for_threshold_above_warn_returns_yellow() {
        let c = Colors::new(true);
        let t = Threshold::Check {
            value: 75.0,
            warn: 70.0,
            crit: 90.0,
        };
        assert_eq!(color_for_threshold(&t, &c), c.yellow);
    }

    #[test]
    /// `color_for_threshold_above_crit_returns_red()` asserts that a value >= crit returns red
    ///
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
