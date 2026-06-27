use std::thread;
use std::time::Duration;

use super::prelude::*;
use crate::constants::{CPU_CRIT_PCT, CPU_WARN_PCT};
use crate::core::utils::read_first_line;

/// `CpuUsageInfo` contains CPU utilization sampled over a configurable sampling period
pub struct CpuUsageInfo {
    pub usage_pct: Option<f64>,
}

/// `CpuUsageService` is used for collecting and rendering CPU utilization
pub struct CpuUsageService {
    pub sample_ms: u64, // Duration of the sampling in milliseconds
}

/// `CpuSnap` contains a single snapshot of aggregate CPU jiffies from `/proc/stat`
struct CpuSnap {
    total: u64,
    idle: u64,
}

/// `CpuUsageService` implements the `Service` trait
impl Service for CpuUsageService {
    type Data = CpuUsageInfo;

    /// `collect()` samples CPU utilization by taking two `/proc/stat` snapshots `sample_ms` apart
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(CpuUsageInfo {
            usage_pct: sample_cpu_usage(self.sample_ms),
        })
    }

    /// `render()` renders CPU utilization with threshold-based color coding
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) {
        let (cpu_str, cpu_thresh) = data.usage_pct.map_or_else(
            || ("n/a".to_string(), Threshold::None),
            |v| {
                (
                    format!("{v:.1}%"),
                    Threshold::Check {
                        value: v,
                        warn: CPU_WARN_PCT,
                        crit: CPU_CRIT_PCT,
                    },
                )
            },
        );

        print_row(label, &cpu_str, &cpu_thresh, c);
    }
}

/// `read_cpu_snap()` reads a single CPU jiffies snapshot from the first line of `/proc/stat`
///
fn read_cpu_snap() -> Option<CpuSnap> {
    let line = read_first_line("/proc/stat")?;
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();

    let f = |i: usize| fields.get(i).copied().unwrap_or(0);

    let (user, nice, system, idle, iowait, irq, softirq, steal) =
        (f(0), f(1), f(2), f(3), f(4), f(5), f(6), f(7));

    Some(CpuSnap {
        total: user + nice + system + idle + iowait + irq + softirq + steal,
        idle: idle + iowait,
    })
}

/// `sample_cpu_usage()` samples CPU utilization over `sample_ms` milliseconds
///
fn sample_cpu_usage(sample_ms: u64) -> Option<f64> {
    let snap1 = read_cpu_snap()?;

    thread::sleep(Duration::from_millis(sample_ms));

    let snap2 = read_cpu_snap()?;

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
