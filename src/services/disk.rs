use std::process;

use super::prelude::*;
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
        let fallback = DiskInfo::default();

        // -k: report sizes in 1K blocks; -P: POSIX portable output format (stable column order)
        let Ok(output) = process::Command::new("df")
            .args(["-kP", &self.mount])
            .output()
        else {
            return Ok(fallback);
        };

        if !output.status.success() {
            return Ok(fallback);
        }

        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
        let Some(line) = stdout.lines().nth(1) else {
            return Ok(fallback);
        };

        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 {
            return Ok(fallback);
        }

        let total_kb: u64 = cols[1].parse().unwrap_or(0);
        let used_kb: u64 = cols[2].parse().unwrap_or(0);

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
                    warn: 80.0,
                    crit: 95.0,
                },
            )
        };

        print_row("  Disk usage:", &disk_str, &disk_thresh, c);
    }
}
