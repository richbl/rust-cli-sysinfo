use super::prelude::*;
use crate::core::utils::read_first_line;

/// `KernelInfo` contains the kernel version read from `/proc/sys/kernel/osrelease`
#[derive(Default)]
pub struct KernelInfo {
    pub version: String,
}

/// `KernelService` is a struct for collecting and rendering the kernel version
pub struct KernelService;

/// `KernelService` implements the `Service` trait
impl Service for KernelService {
    type Data = KernelInfo;

    /// `collect()` reads the kernel version from `/proc/sys/kernel/osrelease`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let version =
            read_first_line("/proc/sys/kernel/osrelease").unwrap_or_else(|| "unknown".into());
        Ok(KernelInfo { version })
    }

    /// `render()` renders the kernel version
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        print_row("  Kernel:", &data.version, &Threshold::None, c);
    }
}
