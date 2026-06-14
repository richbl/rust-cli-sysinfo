use super::prelude::*;
use crate::core::utils::read_first_line;

/// `HostnameInfo` contains the system hostname read from `/proc/sys/kernel/hostname`
pub struct HostnameInfo {
    pub hostname: String,
}

/// `HostnameService` is a struct for collecting and rendering the system hostname
pub struct HostnameService;

/// `HostnameService` implements the `Service` trait
impl Service for HostnameService {
    type Data = HostnameInfo;

    /// `collect()` reads the hostname from `/proc/sys/kernel/hostname`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let hostname =
            read_first_line("/proc/sys/kernel/hostname").unwrap_or_else(|| "unknown".into());
        Ok(HostnameInfo { hostname })
    }

    /// `render()` renders the hostname
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        print_row("  Hostname:", &data.hostname, &Threshold::None, c);
    }
}
