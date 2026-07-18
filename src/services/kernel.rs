use super::prelude::*;

/// `KernelInfo` contains the kernel/OS build version
pub struct KernelInfo {
    pub version: String,
}

/// `KernelService` is a struct for collecting and rendering the kernel version
pub struct KernelService;

/// `KernelService` implements the `Service` trait
impl Service for KernelService {
    type Data = KernelInfo;

    /// `collect()` reads the kernel version
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let version = sysinfo::System::kernel_version()
            .ok_or_else(|| AppError::DataUnavailable("kernel version unavailable".into()))?;
        Ok(KernelInfo { version })
    }

    /// `render()` renders the kernel version
    //
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.version.clone(),
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
            token: "KNL",
            label: "Kernel",
            description: "Kernel/OS build version",
            sort_order: 40,
        },
        Box::new(KernelService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

    /// `collect_returns_ok_with_non_empty_version()` asserts that kernel version collection
    /// succeeds and returns a non-empty version
    ///
    #[test]
    fn collect_returns_ok_with_non_empty_version() {
        let result = KernelService.collect();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.version.is_empty(), "kernel version must not be empty");
    }

    /// `render_does_not_panic()` asserts that rendering kernel version does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = KernelService.collect().unwrap();
        KernelService.render(&data).unwrap();
    }
}
