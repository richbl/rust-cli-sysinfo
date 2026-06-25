use std::ffi::CString;
use std::io;
use std::mem::MaybeUninit;

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

    /// `collect()` uses `statvfs(2)` against the configured mount path and returns usage
    /// statistics
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let c_mount = CString::new(self.mount.as_str())
            .map_err(|_| AppError::Parse(format!("Invalid mount path: {}", self.mount)))?;

        let mut statvfs_buf = MaybeUninit::<libc::statvfs>::uninit();

        // `c_mount` is a valid null-terminated string, and `statvfs_buf` is a valid pointer
        let ret = unsafe { libc::statvfs(c_mount.as_ptr(), statvfs_buf.as_mut_ptr()) };
        if ret != 0 {
            return Err(AppError::Io(io::Error::last_os_error()));
        }

        // `statvfs` initializes the buffer on success (ret == 0)
        let statvfs_buf = unsafe { statvfs_buf.assume_init() };

        // `f_frsize` is the fundamental file system block size
        let total_bytes = statvfs_buf.f_blocks * statvfs_buf.f_frsize;
        let free_bytes = statvfs_buf.f_bfree * statvfs_buf.f_frsize;
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

    /// `render()` renders disk usage as a percentage with used/total sizes and threshold-based
    /// color coding
    ///
    fn render(&self, label: &str, disk: &Self::Data, c: &Colors) {
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

        print_row(label, &disk_str, &disk_thresh, c);
    }
}
