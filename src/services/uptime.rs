use super::prelude::*;
use crate::core::utils::read_first_line;
use crate::presentation::format::format_uptime;

/// `UptimeInfo` contains the system uptime in seconds read from `/proc/uptime`
pub struct UptimeInfo {
    pub uptime_secs: Option<u64>,
}

/// `UptimeService` is a struct for collecting and rendering system uptime
pub struct UptimeService;

/// `UptimeService` implements the `Service` trait
impl Service for UptimeService {
    type Data = UptimeInfo;

    /// `collect()` reads the uptime in seconds from `/proc/uptime`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let uptime_secs = read_first_line("/proc/uptime").and_then(|raw| {
            raw.split_whitespace()
                .next()?
                .split('.')
                .next()?
                .parse()
                .ok()
        });

        Ok(UptimeInfo { uptime_secs })
    }

    /// `render()` renders system uptime formatted as `DDDd:HHh:MMm:SSs`
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) {
        let uptime_str = data
            .uptime_secs
            .map_or_else(|| "unknown".to_string(), format_uptime);

        print_row(label, &uptime_str, &Threshold::None, c);
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    #[test]
    /// `collect_returns_ok_with_some_uptime()` asserts that uptime collection succeeds and returns
    /// uptime value on Linux
    ///
    fn collect_returns_ok_with_some_uptime() {
        let result = UptimeService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(
            data.uptime_secs.is_some(),
            "uptime_secs must be Some on a running Linux system"
        );
    }

    #[test]
    /// `uptime_is_positive()` asserts that the system uptime is positive
    ///
    fn uptime_is_positive() {
        let data = UptimeService.collect().unwrap();
        assert!(
            data.uptime_secs.unwrap() > 0,
            "uptime must be > 0 on a running system"
        );
    }

    #[test]
    /// `render_does_not_panic()` asserts that rendering uptime does not panic
    ///
    fn render_does_not_panic() {
        let data = UptimeService.collect().unwrap();
        UptimeService.render("  Uptime:", &data, &Colors::new(false));
    }
}
