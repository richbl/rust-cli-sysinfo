use super::prelude::*;
use crate::core::utils::read_first_line;

/// `KernelInfo` contains the kernel version read from `/proc/sys/kernel/osrelease`
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

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    #[test]
    /// `collect_returns_ok_with_non_empty_version()` asserts that kernel version collection
    /// succeeds and returns a non-empty version
    ///
    fn collect_returns_ok_with_non_empty_version() {
        let result = KernelService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(
            !data.version.is_empty(),
            "kernel version must not be empty on Linux"
        );
    }

    #[test]
    /// `kernel_version_contains_dot_separator()` asserts that kernel version contains at least one
    /// dot separator
    ///
    fn kernel_version_contains_dot_separator() {
        // A kernel version string always contains at least one '.' (e.g. "6.1.0")
        let data = KernelService.collect().unwrap();
        assert!(
            data.version.contains('.'),
            "kernel version '{:?}' expected to contain '.'",
            data.version
        );
    }

    #[test]
    /// `render_does_not_panic()` asserts that rendering kernel version does not panic
    ///
    fn render_does_not_panic() {
        let data = KernelService.collect().unwrap();
        KernelService.render(&data, &Colors::new(false));
    }
}
