use std::fs;
use std::io::{self, BufRead};

use super::prelude::*;
use crate::constants::{MEM_CRIT_PCT, MEM_WARN_PCT};

/// `MemInfo` contains memory usage metrics parsed from `/proc/meminfo`
pub struct MemInfo {
    pub total: u64,
    pub used: u64,
    pub pct: f64,
}

/// `MemoryService` is a struct for collecting and rendering memory usage
pub struct MemoryService;

/// `parse_kb()` parses a kilobyte value from a `/proc/meminfo` line (e.g. `"MemTotal: 8192 kB"`)
///
fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Raw field values parsed directly from `/proc/meminfo` before validation or fallback logic
struct RawMemInfo {
    total: Option<u64>,
    available: Option<u64>,
    free: Option<u64>,
}

/// `read_meminfo()` reads `MemTotal`, `MemAvailable`, and `MemFree` from `/proc/meminfo`
/// returning each as `Option<u64>` so the caller can apply fallback logic
///
fn read_meminfo() -> Result<RawMemInfo, AppError> {
    let mut raw = RawMemInfo {
        total: None,
        available: None,
        free: None,
    };

    let file = fs::File::open("/proc/meminfo")?;

    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("MemTotal:") {
            raw.total = Some(parse_kb(&line));
        } else if line.starts_with("MemAvailable:") {
            raw.available = Some(parse_kb(&line));
        } else if line.starts_with("MemFree:") {
            raw.free = Some(parse_kb(&line));
        }

        if raw.total.is_some() && raw.available.is_some() && raw.free.is_some() {
            break;
        }
    }

    Ok(raw)
}

/// `MemoryService` implements the `Service` trait
impl Service for MemoryService {
    type Data = MemInfo;

    /// `collect()` delegates file parsing to `read_meminfo()` then resolves
    /// optional fields and computes usage
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let raw = read_meminfo()?;

        let total = raw.total.ok_or_else(|| {
            AppError::DataUnavailable("MemTotal not found in /proc/meminfo".into())
        })?;

        // Fall back to MemFree when MemAvailable is absent (kernels < 3.14)
        let avail_kb = raw.available.unwrap_or(raw.free.ok_or_else(|| {
            AppError::DataUnavailable(
                "Neither MemAvailable nor MemFree found in /proc/meminfo".into(),
            )
        })?);

        let used = total.saturating_sub(avail_kb);

        #[allow(clippy::cast_precision_loss)]
        let pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemInfo { total, used, pct })
    }

    /// `render()` renders memory usage as a percentage with used/total in MB and threshold-based
    /// color coding
    ///
    fn render(&self, mem: &Self::Data) -> Result<RenderedRow, AppError> {
        let mem_str = format!(
            "{:.1}% ({}M/{}M)",
            mem.pct,
            mem.used / 1024,
            mem.total / 1024
        );

        Ok(RenderedRow {
            value: mem_str,
            threshold: Threshold::Check {
                value: mem.pct,
                warn: MEM_WARN_PCT,
                crit: MEM_CRIT_PCT,
            },
        })
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "RAMU",
            label: "Memory usage",
            description: "Memory usage % (Used/Total)",
            sort_order: 110,
        },
        Box::new(MemoryService),
    )
}
