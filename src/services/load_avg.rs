use super::prelude::*;
use crate::core::utils::read_first_line;

/// `LoadAvgInfo` contains the system load averages parsed from `/proc/loadavg`
#[derive(Default)]
pub struct LoadAvgInfo {
    pub loadavg: Option<(f64, f64, f64)>,
}

/// `LoadAvgService` is a struct for collecting and rendering system load averages
pub struct LoadAvgService;

/// `LoadAvgService` implements the `Service` trait
impl Service for LoadAvgService {
    type Data = LoadAvgInfo;

    /// `collect()` reads the 1m, 5m, and 15m load averages from `/proc/loadavg`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(LoadAvgInfo {
            loadavg: read_loadavg(),
        })
    }

    /// `render()` renders load averages as a single row
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        let load_str = data.loadavg.map_or_else(
            || "n/a".to_string(),
            |(l1, l5, l15)| format!("{l1:.2}, {l5:.2}, {l15:.2} (1m, 5m, 15m)"),
        );

        print_row("  Load averages:", &load_str, &Threshold::None, c);
    }
}

/// `read_loadavg()` reads the 1m, 5m, and 15m load averages from `/proc/loadavg`
///
fn read_loadavg() -> Option<(f64, f64, f64)> {
    let line = read_first_line("/proc/loadavg")?;
    let mut parts = line.split_whitespace();

    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}
