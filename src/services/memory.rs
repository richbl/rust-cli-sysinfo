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
        let mut total_kb = None;
        let mut available_kb = None;
        let mut free_kb = None;

        let parse_kb = |line: &str| -> u64 {
            line.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        };

        let file = fs::File::open("/proc/meminfo").map_err(AppError::Io)?;

        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                total_kb = Some(parse_kb(&line));
            } else if line.starts_with("MemAvailable:") {
                available_kb = Some(parse_kb(&line));
            } else if line.starts_with("MemFree:") {
                free_kb = Some(parse_kb(&line));
            }

            // Break early only if all fields have been successfully located
            if total_kb.is_some() && available_kb.is_some() && free_kb.is_some() {
                break;
            }
        }

        let total_kb = total_kb.ok_or_else(|| {
            AppError::DataUnavailable("MemTotal not found in /proc/meminfo".into())
        })?;

        // Fall back to MemFree when MemAvailable is absent (kernels < 3.14)
        let avail_kb = available_kb.unwrap_or(free_kb.ok_or_else(|| {
            AppError::DataUnavailable(
                "Neither MemAvailable nor MemFree found in /proc/meminfo".into(),
            )
        })?);

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
