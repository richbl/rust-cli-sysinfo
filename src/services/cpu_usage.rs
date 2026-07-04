use std::thread;
use std::time::Duration;

use super::prelude::*;
use crate::constants::{CPU_CRIT_PCT, CPU_WARN_PCT};
use crate::core::utils::read_first_line;

/// `CpuUsageInfo` contains CPU utilization sampled over a configurable sampling period
pub struct CpuUsageInfo {
    pub usage_pct: f64,
}

/// `CpuUsageService` is used for collecting and rendering CPU utilization
pub struct CpuUsageService {
    pub sample_ms: u64, // Duration of the sampling in milliseconds
}

/// `CpuUsageService` constructor is used for creating a new `CpuUsageService`
impl CpuUsageService {
    /// `new()` creates a new `CpuUsageService`
    ///
    pub fn new(ctx: &ServiceContext) -> Self {
        Self {
            sample_ms: ctx.cpu_sample_ms,
        }
    }
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
        let usage_pct = sample_cpu_usage(self.sample_ms)?;
        Ok(CpuUsageInfo { usage_pct })
    }

    /// `render()` renders CPU utilization with threshold-based color coding
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        let cpu_str = format!("{:.1}%", data.usage_pct);
        let cpu_thresh = Threshold::Check {
            value: data.usage_pct,
            warn: CPU_WARN_PCT,
            crit: CPU_CRIT_PCT,
        };

        print_row(label, &cpu_str, &cpu_thresh, c);
        Ok(())
    }
}

/// `read_cpu_snap()` reads a single snapshot of aggregate CPU jiffies from `/proc/stat`
///
fn read_cpu_snap() -> Result<CpuSnap, AppError> {
    let line = read_first_line("/proc/stat")?;

    // Allocate an array on the stack for the first 8 CPU metrics
    let mut fields = [0u64; 8];

    for (i, val) in line.split_whitespace().skip(1).take(8).enumerate() {
        fields[i] = val.parse::<u64>().unwrap_or(0);
    }

    let user = fields[0];
    let nice = fields[1];
    let system = fields[2];
    let idle = fields[3];
    let iowait = fields[4];
    let irq = fields[5];
    let softirq = fields[6];
    let steal = fields[7];

    Ok(CpuSnap {
        total: user + nice + system + idle + iowait + irq + softirq + steal,
        idle: idle + iowait,
    })
}

/// `sample_cpu_usage()` samples CPU utilization over `sample_ms` milliseconds
///
fn sample_cpu_usage(sample_ms: u64) -> Result<f64, AppError> {
    let snap1 = read_cpu_snap()?;

    thread::sleep(Duration::from_millis(sample_ms));

    let snap2 = read_cpu_snap()?;

    let d_total = snap2.total.saturating_sub(snap1.total);
    let d_idle = snap2.idle.saturating_sub(snap1.idle);

    if d_total == 0 {
        return Ok(0.0);
    }

    let d_used = d_total.saturating_sub(d_idle);

    #[allow(clippy::cast_precision_loss)]
    let pct = (d_used as f64 / d_total as f64) * 100.0;

    Ok(pct)
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "CPUU",
            label: "CPU usage",
            description: "CPU usage %",
            sort_order: 100,
        },
        Box::new(CpuUsageService::new(ctx)),
    )
}
