use std::process;

use super::prelude::*;
use crate::constants::{DISK_CRIT_PCT, DISK_WARN_PCT};
use crate::presentation::format::format_size;

/// `DiskInfo` is a struct containing disk usage metrics for a single mount point
#[derive(Default)]
pub struct DiskInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub pct: f64,
}

/// `DiskService` is a struct for collecting and rendering disk usage for a given mount path
pub struct DiskService {
    pub mount: String,
}

/// `DiskService` implements the `Service` trait
impl Service for DiskService {
    type Data = DiskInfo;

    /// `collect()` runs `df -kP` against the configured mount path and returns usage statistics
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        // -k: report sizes in 1K blocks; -P: POSIX portable output format (stable column order)
        let output = process::Command::new("df")
            .args(["-kP", &self.mount])
            .output()
            .map_err(AppError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::DataUnavailable(format!(
                "df failed for {}: {}",
                self.mount,
                stderr.trim()
            )));
        }

        let stdout = std::str::from_utf8(&output.stdout)
            .map_err(|e| AppError::Parse(format!("Invalid UTF-8 in df output: {e}")))?;
        let line = stdout
            .lines()
            .nth(1)
            .ok_or_else(|| AppError::Parse("df output missing data line".into()))?;

        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 {
            return Err(AppError::Parse(format!(
                "Unexpected df output format: {line}"
            )));
        }

        let total_kb: u64 = cols[1]
            .parse()
            .map_err(|_| AppError::Parse(format!("Failed to parse df total_kb: {}", cols[1])))?;
        let used_kb: u64 = cols[2]
            .parse()
            .map_err(|_| AppError::Parse(format!("Failed to parse df used_kb: {}", cols[2])))?;

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
    /// `render()` renders disk usage as a percentage with used/total sizes and threshold-based
    /// color coding
    ///
    fn render(&self, disk: &Self::Data, c: &Colors) {
        let (disk_str, disk_thresh) = if disk.total_kb == 0 {
            ("n/a".to_string(), Threshold::None)
        } else {
            let text = format!(
                "{:.1}% ({}/{}) of {}",
                disk.pct,
                format_size(disk.used_kb),
                format_size(disk.total_kb),
                self.mount
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

        print_row("  Disk usage:", &disk_str, &disk_thresh, c);
    }
}
