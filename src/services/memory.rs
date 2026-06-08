use std::fs;
use std::io::{self, BufRead};

use super::Service;
use crate::core::error::AppError;
use crate::presentation::colors::{Colors, Threshold};
use crate::presentation::format::print_row;

/// Memory usage metrics parsed from `/proc/meminfo`
pub struct MemInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub pct: f64,
}

/// Service for collecting and rendering physical memory usage
pub struct MemoryService;

/// Collects and renders memory usage
impl Service for MemoryService {
    type Data = MemInfo;

    /// `collect()` reads `MemTotal`, `MemAvailable`, and `MemFree` from `/proc/meminfo` and
    /// computes usage
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut total_kb = 0;
        let mut available_kb = None;
        let mut free_kb = 0;

        // Parse MemTotal, MemAvailable, and MemFree
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
            "{:.1}% ({}MB/{}MB)",
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
