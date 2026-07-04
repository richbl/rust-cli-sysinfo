use super::prelude::*;
use crate::core::utils::read_first_line;

/// `LoadAvgInfo` contains the system load averages parsed from `/proc/loadavg`
pub struct LoadAvgInfo {
    pub loadavg: (f64, f64, f64),
}

/// `LoadAvgService` is a struct for collecting and rendering system load averages
pub struct LoadAvgService;

/// `LoadAvgService` implements the `Service` trait
impl Service for LoadAvgService {
    type Data = LoadAvgInfo;

    /// `collect()` reads the 1m, 5m, and 15m load averages from `/proc/loadavg`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let line = read_first_line("/proc/loadavg")?;
        let mut parts = line.split_whitespace();

        let l1 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::DataUnavailable("failed to parse 1m loadavg".into()))?;
        let l5 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::DataUnavailable("failed to parse 5m loadavg".into()))?;
        let l15 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| AppError::DataUnavailable("failed to parse 15m loadavg".into()))?;

        Ok(LoadAvgInfo {
            loadavg: (l1, l5, l15),
        })
    }

    /// `render()` renders load averages as a single row
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        let (l1, l5, l15) = data.loadavg;
        Ok(RenderedRow {
            value: format!("{l1:.2}, {l5:.2}, {l15:.2} (1m, 5m, 15m)"),
            threshold: Threshold::None,
        })
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "LOAD",
            label: "Load averages",
            description: "Load averages (1m, 5m, 15m)",
            sort_order: 90,
        },
        Box::new(LoadAvgService),
    )
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    /// `collect_returns_ok_with_some_loadavg()` asserts that load average collection succeeds and
    /// returns load averages on Linux
    ///
    #[test]
    fn collect_returns_ok_with_some_loadavg() {
        let result = LoadAvgService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(
            data.loadavg.0 >= 0.0,
            "loadavg must be collected on a running system"
        );
    }

    /// `all_three_averages_are_non_negative()` asserts that all three load averages are
    /// non-negative
    ///
    #[test]
    fn all_three_averages_are_non_negative() {
        let data = LoadAvgService.collect().unwrap();
        let (l1, l5, l15) = data.loadavg;
        assert!(l1 >= 0.0, "1m load average must be non-negative");
        assert!(l5 >= 0.0, "5m load average must be non-negative");
        assert!(l15 >= 0.0, "15m load average must be non-negative");
    }

    /// `render_does_not_panic()` asserts that rendering load averages does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = LoadAvgService.collect().unwrap();
        LoadAvgService.render(&data).unwrap();
    }
}
