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
