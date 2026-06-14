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
    fn render(&self, data: &Self::Data, c: &Colors) {
        let uptime_str = data
            .uptime_secs
            .map_or_else(|| "unknown".to_string(), format_uptime);

        print_row("  Uptime:", &uptime_str, &Threshold::None, c);
    }
}
