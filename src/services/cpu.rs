use std::thread;
use std::time::Duration;

use super::Service;
use crate::core::error::AppError;
use crate::core::utils::read_first_line;
use crate::presentation::colors::{Colors, Threshold};
use crate::presentation::format::print_row;

/// CPU metrics collected from the system
pub struct CpuInfo {
    pub loadavg: Option<(f64, f64, f64)>, // 1m, 5m, and 15m load averages from `/proc/loadavg`
    pub usage_pct: Option<f64>, // Instantaneous CPU utilization percentage sampled over a fixed window
}

/// Service for collecting and rendering CPU metrics
pub struct CpuService {
    pub sample_ms: u64, // Duration of the CPU sampling window in milliseconds
}

/// A single snapshot of aggregate CPU jiffies (you heard that right) from `/proc/stat`
struct CpuSnap {
    total: u64, // Sum of all CPU time fields
    idle: u64,  // Sum of idle + iowait time (both count as "not doing work")
}

/// `read_cpu_snap()` reads a single CPU jiffies snapshot from the first line of `/proc/stat`
///
fn read_cpu_snap() -> Option<CpuSnap> {
    let line = read_first_line("/proc/stat")?;
    let mut fields = line
        .split_whitespace()
        .skip(1) // skip the "cpu" label
        .filter_map(|v| v.parse::<u64>().ok());

    let user = fields.next().unwrap_or(0);
    let nice = fields.next().unwrap_or(0);
    let system = fields.next().unwrap_or(0);
    let idle = fields.next().unwrap_or(0);
    let iowait = fields.next().unwrap_or(0);
    let irq = fields.next().unwrap_or(0);
    let softirq = fields.next().unwrap_or(0);
    let steal = fields.next().unwrap_or(0);

    Some(CpuSnap {
        total: user + nice + system + idle + iowait + irq + softirq + steal,
        idle: idle + iowait,
    })
}

/// `read_loadavg()` reads the 1m, 5m, and 15m load averages from `/proc/loadavg`
///
fn read_loadavg() -> Option<(f64, f64, f64)> {
    let line = read_first_line("/proc/loadavg")?;
    let mut parts = line.split_whitespace();

    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

/// `sample_cpu_usage()` samples CPU utilization by taking two `/proc/stat` snapshots `sample_ms`
/// apart
///
fn sample_cpu_usage(sample_ms: u64) -> Option<f64> {
    let snap1 = read_cpu_snap()?;

    thread::sleep(Duration::from_millis(sample_ms));

    let snap2 = read_cpu_snap()?;

    // Compute how many jiffies elapsed and how many were idle in the interval
    let d_total = snap2.total.saturating_sub(snap1.total);
    let d_idle = snap2.idle.saturating_sub(snap1.idle);

    if d_total == 0 {
        return Some(0.0);
    }

    let d_used = d_total.saturating_sub(d_idle);

    #[allow(clippy::cast_precision_loss)]
    let pct = (d_used as f64 / d_total as f64) * 100.0;

    Some(pct)
}

/// Collects and renders CPU metrics
impl Service for CpuService {
    type Data = CpuInfo;

    /// `collect()` collects load averages and samples CPU utilization over [`CpuService::sample_ms`]
    /// milliseconds
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(CpuInfo {
            loadavg: read_loadavg(),
            usage_pct: sample_cpu_usage(self.sample_ms),
        })
    }

    /// `render()` renders CPU load averages and utilization with threshold-based color coding
    ///
    fn render(&self, cpu: &Self::Data, c: &Colors) {
        let load_str = cpu.loadavg.map_or_else(
            || "n/a".to_string(),
            |(l1, l5, l15)| format!("{l1:.2}, {l5:.2}, {l15:.2} (1m, 5m, 15m)"),
        );

        print_row("  Load averages:", &load_str, &Threshold::None, c);

        let (cpu_str, cpu_thresh) = cpu.usage_pct.map_or_else(
            || ("n/a".to_string(), Threshold::None),
            |v| {
                (
                    format!("{v:.1}%"),
                    Threshold::Check {
                        value: v,
                        warn: 70.0,
                        crit: 90.0,
                    },
                )
            },
        );
        print_row("  CPU usage:", &cpu_str, &cpu_thresh, c);
    }
}
