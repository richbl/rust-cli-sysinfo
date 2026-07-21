use super::prelude::*;
use crate::constants::{KB_PER_GB, MEM_CRIT_PCT, MEM_WARN_PCT};

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

    /// `render()` renders memory usage as a percentage with used/total in GiB and threshold-based
    /// color coding
    ///
    fn render(&self, mem: &Self::Data) -> Result<RenderedRow, AppError> {
        // Convert to tenths of GiB using integer arithmetic to avoid casting large
        // integers to f64 and triggering potential precision-loss
        let used_tenths =
            (u128::from(mem.used) * 10 + u128::from(KB_PER_GB) / 2) / u128::from(KB_PER_GB);
        let total_tenths =
            (u128::from(mem.total) * 10 + u128::from(KB_PER_GB) / 2) / u128::from(KB_PER_GB);

        // Split into whole and fractional (tenths) parts and format without
        // creating f64 values for the sizes.
        let used_whole = used_tenths / 10;
        let used_frac = used_tenths % 10;
        let total_whole = total_tenths / 10;
        let total_frac = total_tenths % 10;

        let mem_str = format!(
            "{:.1}% ({}.{}/{}.{} GiB)",
            mem.pct, used_whole, used_frac, total_whole, total_frac
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `collect_returns_ok_and_nonzero_total()` asserts that memory collection succeeds and
    /// returns non-zero total memory on a running system
    ///
    #[test]
    fn collect_returns_ok_and_nonzero_total() {
        let result = MemoryService.collect();
        assert!(result.is_ok());
        let mem = result.unwrap();
        assert!(mem.total > 0);
    }

    /// `render_formats_in_gib()` asserts that rendering memory usage formats used and total
    /// in GiB with one decimal place
    ///
    #[test]
    fn render_formats_in_gib() {
        let mem = MemInfo {
            total: 28_351_488, // ~27.0 GiB
            used: 4_346_880,   // ~4.145 GiB -> 4.1 GiB
            pct: 15.3,
        };

        let row = MemoryService.render(&mem).unwrap();
        assert_eq!(row.value, "15.3% (4.1/27.0 GiB)");
    }
}
