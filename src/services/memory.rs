use super::prelude::*;
use crate::constants::{MEM_CRIT_PCT, MEM_WARN_PCT};

/// `MemInfo` contains memory usage metrics
///
pub struct MemInfo {
    pub total: u64,
    pub used: u64,
    pub pct: f64,
}

/// `MemoryService` is a struct for collecting and rendering memory usage
pub struct MemoryService;

/// `MemoryService` implements the `Service` trait
impl Service for MemoryService {
    type Data = MemInfo;

    /// `collect()` creates a fresh, minimal `sysinfo::System` and refreshes only memory data,
    /// then converts `sysinfo`'s byte-denominated values to KB
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();

        let total = sys.total_memory() / 1024;
        let used = sys.used_memory() / 1024;

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
