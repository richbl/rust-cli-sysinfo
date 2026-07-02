use std::path::PathBuf;

use super::prelude::*;
use crate::constants::{DISK_CRIT_PCT, DISK_WARN_PCT};
use crate::presentation::format::format_size;

/// `DiskInfo` contains disk usage metrics for a single mount point
#[derive(Default)]
pub struct DiskInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub pct: f64,
}

/// `DiskService` collects and renders disk usage for a given mount path
pub struct DiskService {
    pub mount: PathBuf,
}

/// `DiskService` is a struct for collecting and rendering disk usage
impl DiskService {
    /// `new()` creates a new `DiskService`
    ///
    pub fn new(ctx: &ServiceContext) -> Self {
        Self {
            mount: ctx.disk_mount.clone(),
        }
    }
}

/// `AppError` is the application-level error type returned by all service `collect()` calls
impl From<rustix::io::Errno> for AppError {
    /// `from()` converts `rustix::io::Errno` into `AppError`
    ///
    fn from(e: rustix::io::Errno) -> Self {
        Self::Io(std::io::Error::from(e))
    }
}

/// `DiskService` implements the `Service` trait
impl Service for DiskService {
    type Data = DiskInfo;

    /// `collect()` uses `statvfs(2)` against the configured mount path
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        // Map and convert std::io::Error into AppError
        let stat = rustix::fs::statvfs(&self.mount)?;

        let total_bytes = stat.f_blocks * stat.f_frsize;
        let free_bytes = stat.f_bfree * stat.f_frsize;
        let total_kb = total_bytes / 1024;

        // `df` calculates used as: total - free
        let used_kb = total_kb.saturating_sub(free_bytes / 1024);

        #[allow(clippy::cast_precision_loss)]
        let pct = if total_kb > 0 {
            (used_kb as f64 / total_kb as f64) * 100.0
        } else {
            0.0
        };

        Ok(DiskInfo {
            total_kb,
            used_kb,
            pct,
        })
    }

    /// `render()` renders disk usage as a percentage with used/total sizes
    ///
    fn render(&self, label: &str, disk: &Self::Data, c: &Colors) -> Result<(), AppError> {
        let (disk_str, disk_thresh) = if disk.total_kb == 0 {
            ("n/a".to_string(), Threshold::None)
        } else {
            // display() handles formatting potential non-UTF-8 characters
            let text = format!(
                "{:.1}% ({}/{}) of {}",
                disk.pct,
                format_size(disk.used_kb),
                format_size(disk.total_kb),
                self.mount.display()
            );
            (
                text,
                Threshold::Check {
                    value: disk.pct,
                    warn: DISK_WARN_PCT,
                    crit: DISK_CRIT_PCT,
                },
            )
        };

        print_row(label, &disk_str, &disk_thresh, c);
        Ok(())
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "DSKU",
            label: "Disk usage",
            description: "Disk usage % (Used/Total)",
            sort_order: 9,
        },
        Box::new(DiskService::new(ctx)),
    )
}
