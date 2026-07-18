use super::prelude::*;
use crate::presentation::format::format_uptime;

/// `UptimeInfo` contains the system uptime in seconds
pub struct UptimeInfo {
    pub uptime_secs: u64,
}

/// `UptimeService` is a struct for collecting and rendering system uptime
pub struct UptimeService;

/// `UptimeService` implements the `Service` trait
impl Service for UptimeService {
    type Data = UptimeInfo;

    /// `collect()` reads system uptime in seconds
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(UptimeInfo {
            uptime_secs: sysinfo::System::uptime(),
        })
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
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

    /// `collect_returns_ok_with_some_uptime()` asserts that uptime collection succeeds and
    /// returns a positive value on a running system
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
