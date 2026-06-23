use super::colors::{Colors, Threshold};

/// `print_row()` prints a left-aligned label/value row, coloring the value
///
pub fn print_row(label: &str, value: &str, threshold: &Threshold, c: &Colors) {
    let color = match threshold {
        Threshold::Check {
            value: v,
            warn: _,
            crit,
        } if v >= crit => c.red,
        Threshold::Check {
            value: v,
            warn,
            crit: _,
        } if v >= warn => c.yellow,
        Threshold::Check { .. } => c.green,
        Threshold::None => c.reset,
    };

    println!("{:<16} {}{}{}", label, color, value, c.reset);
}

/// `format_uptime()` formats a duration in seconds as `DDDd:HHh:MMm:SSs`
///
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    format!("{days:03}d:{hours:02}h:{mins:02}m:{secs:02}s")
}

/// `format_size()` formats a size given in kilobytes as a human-readable string with a T/G/M/K
/// suffix
///
pub fn format_size(kb: u64) -> String {
    // Divisors are powers of 1024 (all expressed in KB)

    #[allow(clippy::cast_precision_loss)]
    let k = kb as f64;

    if k >= 1_073_741_824.0 {
        format!("{:.1}T", k / 1_073_741_824.0)
    } else if k >= 1_048_576.0 {
        format!("{:.1}G", k / 1_048_576.0)
    } else if k >= 1_024.0 {
        format!("{:.1}M", k / 1_024.0)
    } else {
        format!("{kb}K")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // format_uptime test
    //
    // Boundary table:
    //   0          → 000d:00h:00m:00s
    //   1          → 000d:00h:00m:01s
    //   59         → 000d:00h:00m:59s
    //   60         → 000d:00h:01m:00s
    //   3_599      → 000d:00h:59m:59s
    //   3_600      → 000d:01h:00m:00s
    //   86_399     → 000d:23h:59m:59s
    //   86_400     → 001d:00h:00m:00s
    //   93_784     → 001d:02h:03m:04s  (1d+2h+3m+4s)
    //   31_536_000 → 365d:00h:00m:00s  (1 year)

    #[test]
    /// `uptime_zero_seconds_formats_all_zeros` asserts that a duration of 0 seconds is formatted
    /// correctly
    ///
    fn uptime_zero_seconds_formats_all_zeros() {
        assert_eq!(format_uptime(0), "000d:00h:00m:00s");
    }

    #[test]
    /// `uptime_one_second` asserts that a duration of 1 second is formatted correctly
    ///
    fn uptime_one_second() {
        assert_eq!(format_uptime(1), "000d:00h:00m:01s");
    }

    #[test]
    /// `uptime_59_seconds_stays_in_seconds_field` asserts that a duration of 59 seconds is formatted
    /// correctly
    ///
    fn uptime_59_seconds_stays_in_seconds_field() {
        assert_eq!(format_uptime(59), "000d:00h:00m:59s");
    }

    #[test]
    /// `uptime_60_seconds_rolls_into_minutes` asserts that a duration of 60 seconds is formatted
    /// correctly when it rolls into minutes
    ///
    fn uptime_60_seconds_rolls_into_minutes() {
        assert_eq!(format_uptime(60), "000d:00h:01m:00s");
    }

    #[test]
    /// `uptime_one_minute_before_hour_boundary` asserts that a duration of 3,599 seconds is formatted
    /// correctly
    ///
    fn uptime_one_minute_before_hour_boundary() {
        assert_eq!(format_uptime(3_599), "000d:00h:59m:59s");
    }

    #[test]
    /// `uptime_3600_seconds_rolls_into_hours` asserts that a duration of 3,600 seconds is formatted
    /// correctly (rolls into hours)
    ///
    fn uptime_3600_seconds_rolls_into_hours() {
        assert_eq!(format_uptime(3_600), "000d:01h:00m:00s");
    }

    #[test]
    /// `uptime_one_second_before_day_boundary` asserts that a duration of 86,399 seconds is formatted
    /// correctly
    ///
    fn uptime_one_second_before_day_boundary() {
        assert_eq!(format_uptime(86_399), "000d:23h:59m:59s");
    }

    #[test]
    /// `uptime_86400_seconds_rolls_into_days()` asserts that exactly 24 hours rolls into the days
    /// field
    ///
    fn uptime_86400_seconds_rolls_into_days() {
        assert_eq!(format_uptime(86_400), "001d:00h:00m:00s");
    }

    #[test]
    /// `uptime_mixed_all_units()` asserts that a duration with days, hours, minutes, and seconds is
    /// formatted correctly
    ///
    fn uptime_mixed_all_units() {
        // 1d + 2h + 3m + 4s = 86400 + 7200 + 180 + 4 = 93784
        assert_eq!(format_uptime(93_784), "001d:02h:03m:04s");
    }

    #[test]
    /// `uptime_large_day_count_pads_to_three_digits()` asserts that days field pads to three digits
    /// and does not truncate for large values
    ///
    fn uptime_large_day_count_pads_to_three_digits() {
        // 365 days: days field should NOT truncate
        assert_eq!(format_uptime(365 * 86_400), "365d:00h:00m:00s");
    }

    // --- format_size ---
    // Boundary table (all values in KB):
    //   0              → "0K"
    //   1              → "1K"
    //   1_023          → "1023K"
    //   1_024          → "1.0M"    (1024 KB = 1 MB)
    //   1_536          → "1.5M"    (1536 KB = 1.5 MB)
    //   1_048_575      → "1024.0M" → actually "1024.0M"... wait
    //   1_048_576      → "1.0G"    (1024^2 KB = 1 GB)
    //   1_073_741_823  → last value before TB
    //   1_073_741_824  → "1.0T"    (1024^3 KB = 1 TB)

    #[test]
    /// `size_zero_kb_formats_as_zero_k()` asserts that 0 KB formats as "0K"
    ///
    fn size_zero_kb_formats_as_zero_k() {
        assert_eq!(format_size(0), "0K");
    }

    #[test]
    /// `size_one_kb()` asserts that 1 KB formats as "1K"
    ///
    fn size_one_kb() {
        assert_eq!(format_size(1), "1K");
    }

    #[test]
    /// `size_1023_kb_stays_in_k_suffix()` asserts that 1023 KB stays in KB scale
    ///
    fn size_1023_kb_stays_in_k_suffix() {
        assert_eq!(format_size(1_023), "1023K");
    }

    #[test]
    /// `size_1024_kb_crosses_into_megabytes()` asserts that 1024 KB crosses into MB scale
    ///
    fn size_1024_kb_crosses_into_megabytes() {
        assert_eq!(format_size(1_024), "1.0M");
    }

    #[test]
    /// `size_fractional_megabytes()` asserts that fractional MB is formatted with one decimal
    /// place
    ///
    fn size_fractional_megabytes() {
        // 1536 KB = 1.5 MB
        assert_eq!(format_size(1_536), "1.5M");
    }

    #[test]
    /// `size_1_gb_boundary()` asserts that 1 GB boundary is formatted correctly
    ///
    fn size_1_gb_boundary() {
        // 1024 * 1024 KB = 1 GB
        assert_eq!(format_size(1_048_576), "1.0G");
    }

    #[test]
    /// `size_fractional_gigabytes()` asserts that fractional GB is formatted with one decimal
    /// place
    ///
    fn size_fractional_gigabytes() {
        // 1.5 GB = 1536 * 1024 KB
        assert_eq!(format_size(1_572_864), "1.5G");
    }

    #[test]
    /// `size_1_tb_boundary()` asserts that 1 TB boundary is formatted correctly
    ///
    fn size_1_tb_boundary() {
        // 1024^3 KB = 1 TB
        assert_eq!(format_size(1_073_741_824), "1.0T");
    }

    #[test]
    /// `size_fractional_terabytes()` asserts that fractional TB is formatted with one decimal
    /// place
    ///
    fn size_fractional_terabytes() {
        // 1.5 TB = 1536 * 1024^2 KB
        assert_eq!(format_size(1_610_612_736), "1.5T");
    }

    // --- print_row (not yet unit-testable) ---
    // print_row writes directly to stdout. Once `color_for_threshold()` is
    // extracted as a pure function in Phase 4 (squawk #9), tests should be added
    // here to verify:
    //   - value < warn  → c.green
    //   - value >= warn → c.yellow
    //   - value >= crit → c.red
    //   - Threshold::None → c.reset
}
