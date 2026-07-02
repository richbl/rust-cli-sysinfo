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
    //
    fn collect(&self) -> Result<Self::Data, AppError> {
        let version = read_first_line("/proc/sys/kernel/osrelease")?;
        Ok(KernelInfo { version })
    }

    /// `render()` renders the kernel version
    //
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        print_row(label, &data.version, &Threshold::None, c);
        Ok(())
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "KNL",
            label: "Kernel",
            description: "Linux kernel version",
            sort_order: 4,
        },
        Box::new(KernelService),
    )
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    /// `collect_returns_ok_with_non_empty_version()` asserts that kernel version collection
    /// succeeds and returns a non-empty version
    ///
    #[test]
    fn collect_returns_ok_with_non_empty_version() {
        let result = KernelService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(
            !data.version.is_empty(),
            "kernel version must not be empty on Linux"
        );
    }

    /// `kernel_version_contains_dot_separator()` asserts that kernel version contains at least one
    /// dot separator
    ///
    #[test]
    fn kernel_version_contains_dot_separator() {
        // A kernel version string always contains at least one '.' (e.g. "6.1.0")
        let data = KernelService.collect().unwrap();
        assert!(
            data.version.contains('.'),
            "kernel version '{:?}' expected to contain '.'",
            data.version
        );
    }

    /// `render_does_not_panic()` asserts that rendering kernel version does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = KernelService.collect().unwrap();
        KernelService
            .render("Kernel", &data, &Colors::new(false))
            .unwrap();
    }
}
