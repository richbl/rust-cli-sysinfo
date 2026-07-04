use super::prelude::*;
use crate::core::utils::read_first_line;
use crate::presentation::format::format_uptime;

/// `UptimeInfo` contains the system uptime in seconds read from `/proc/uptime`
pub struct UptimeInfo {
    pub uptime_secs: u64,
}

/// `UptimeService` is a struct for collecting and rendering system uptime
pub struct UptimeService;

/// `UptimeService` implements the `Service` trait
impl Service for UptimeService {
    type Data = UptimeInfo;

    /// `collect()` reads the uptime in seconds from `/proc/uptime`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let raw = read_first_line("/proc/uptime")?;
        let uptime_secs = raw
            .split_whitespace()
            .next()
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| AppError::DataUnavailable("failed to parse uptime".into()))?;

        Ok(UptimeInfo { uptime_secs })
    }

    /// `render()` renders system uptime formatted as `DDDd:HHh:MMm:SSs`
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: format_uptime(data.uptime_secs),
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
            token: "UPT",
            label: "Uptime",
            description: "System uptime",
            sort_order: 80,
        },
        Box::new(UptimeService),
    )
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    /// `collect_returns_ok_with_some_uptime()` asserts that uptime collection succeeds and returns
    /// uptime value on Linux
    ///
    #[test]
    fn collect_returns_ok_with_some_uptime() {
        let result = UptimeService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(
            data.uptime_secs > 0,
            "uptime_secs must be > 0 on a running system"
        );
    }

    /// `uptime_is_positive()` asserts that the system uptime is positive
    ///
    #[test]
    fn uptime_is_positive() {
        let data = UptimeService.collect().unwrap();
        assert!(
            data.uptime_secs > 0,
            "uptime must be > 0 on a running system"
        );
    }

    /// `render_does_not_panic()` asserts that rendering uptime does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = UptimeService.collect().unwrap();
        UptimeService.render(&data).unwrap();
    }
}
