use std::thread;
use std::time::Duration;

use super::prelude::*;
use crate::constants::{CPU_CRIT_PCT, CPU_WARN_PCT};

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

/// `CpuUsageService` implements the `Service` trait
impl Service for CpuUsageService {
    type Data = CpuUsageInfo;

    /// `collect()` samples CPU utilization by taking two `sysinfo` CPU refreshes `sample_ms`
    /// apart
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut sys = sysinfo::System::new();

        sys.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(self.sample_ms));
        sys.refresh_cpu_usage();

        let usage_pct = f64::from(sys.global_cpu_usage());

        Ok(CpuUsageInfo { usage_pct })
    }

    /// `render()` renders CPU utilization with threshold-based color coding
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: format!("{:.1}%", data.usage_pct),
            threshold: Threshold::Check {
                value: data.usage_pct,
                warn: CPU_WARN_PCT,
                crit: CPU_CRIT_PCT,
            },
        })
    }
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
