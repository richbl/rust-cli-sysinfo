use std::fs;
use std::io::{self, BufRead};

use super::prelude::*;

/// `MemInfo` contains memory usage metrics parsed from `/proc/meminfo`
pub struct MemInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub pct: f64,
}

/// `MemoryService` is a struct for collecting and rendering memory usage
pub struct MemoryService;

/// `MemoryService` implements the `Service` trait
impl Service for MemoryService {
    type Data = MemInfo;

    /// `collect()` reads `MemTotal`, `MemAvailable`, and `MemFree` from `/proc/meminfo` and
    /// computes usage
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut total_kb = 0;
        let mut available_kb = None;
        let mut free_kb = 0;

        let parse_kb = |line: &str| -> u64 {
            line.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        };

        if let Ok(file) = fs::File::open("/proc/meminfo") {
            for line in io::BufReader::new(file).lines().map_while(Result::ok) {
                if line.starts_with("MemTotal:") {
                    total_kb = parse_kb(&line);
                } else if line.starts_with("MemAvailable:") {
                    available_kb = Some(parse_kb(&line));
                } else if line.starts_with("MemFree:") {
                    free_kb = parse_kb(&line);
                }

                if total_kb > 0 && available_kb.is_some() && free_kb > 0 {
                    break;
                }
            }
        }

        // Fall back to MemFree when MemAvailable is absent (kernels < 3.14)
        let avail_kb = available_kb.unwrap_or(free_kb);
        let used_kb = total_kb.saturating_sub(avail_kb);

        #[allow(clippy::cast_precision_loss)]
        let pct = if total_kb > 0 {
            (used_kb as f64 / total_kb as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemInfo {
            total_kb,
            used_kb,
            pct,
        })
    }

    /// `render()` renders memory usage as a percentage with used/total in MB and threshold-based
    /// color coding
    ///
    fn render(&self, mem: &Self::Data, c: &Colors) {
        let mem_str = format!(
            "{:.1}% ({}M/{}M)",
            mem.pct,
            mem.used_kb / 1024,
            mem.total_kb / 1024
        );

        print_row(
            "  Memory usage:",
            &mem_str,
            &Threshold::Check {
                value: mem.pct,
                warn: 75.0,
                crit: 90.0,
            },
            c,
        );
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    #[test]
    /// `collect_returns_ok_with_positive_total()` asserts that memory collection succeeds with a
    /// total memory > 0
    ///
    fn collect_returns_ok_with_positive_total() {
        let result = MemoryService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.total_kb > 0, "total memory must be > 0 on Linux");
    }

    #[test]
    /// `used_does_not_exceed_total()` asserts that used memory does not exceed total memory
    ///
    fn used_does_not_exceed_total() {
        let data = MemoryService.collect().unwrap();
        assert!(
            data.used_kb <= data.total_kb,
            "used ({}) must not exceed total ({})",
            data.used_kb,
            data.total_kb
        );
    }

    #[test]
    /// `percentage_is_in_valid_range()` asserts that memory usage percentage is in the range
    /// [0.0, 100.0]
    ///
    fn percentage_is_in_valid_range() {
        let data = MemoryService.collect().unwrap();
        assert!(
            (0.0..=100.0).contains(&data.pct),
            "memory pct {:.1} is outside [0, 100]",
            data.pct
        );
    }

    #[test]
    /// `render_does_not_panic()` asserts that rendering memory info does not panic
    ///
    fn render_does_not_panic() {
        let data = MemoryService.collect().unwrap();
        MemoryService.render(&data, &Colors::new(false));
    }
}
