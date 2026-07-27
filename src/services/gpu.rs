use super::prelude::*;
use crate::constants::{INDENT, LABEL_WIDTH};

/// Deduplicated list of GPU display names
///
pub struct GpuInfo {
    pub models: Vec<String>,
}

/// `GpuService` is a struct for collecting and rendering GPU model name(s)
pub struct GpuService;

/// `GpuService` implements the `Service` trait
impl Service for GpuService {
    type Data = GpuInfo;

    /// `collect()` delegates to the platform-specific implementation selected below
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(GpuInfo {
            models: platform::collect_gpu_models()?,
        })
    }

    /// `render()` renders GPU model name(s)
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        let separator = format!("\n{:width$}", "", width = INDENT.len() + LABEL_WIDTH + 1);
        let value = if data.models.is_empty() {
            "Unknown".to_string()
        } else {
            data.models.join(&separator)
        };
        Ok(RenderedRow {
            value,
            threshold: Threshold::None,
        })
    }
}

// Platform-specific collection lives under `src/services/gpu/`: one file per OS/platform
//
#[cfg(target_os = "linux")]
#[path = "gpu/linux.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "gpu/windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "gpu/macos.rs"]
mod platform;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
compile_error!(
    "the `gpu` service has no implementation for this target; add src/services/gpu/<platform>.rs and wire it in via #[cfg] in src/services/gpu.rs"
);

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "GPU",
            label: "GPU(s)",
            description: "GPU model(s)",
            sort_order: 20,
        },
        Box::new(GpuService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod tests {
    use super::*;

    /// `collect_returns_ok_on_supported_os()` asserts that GPU collection succeeds (an empty
    /// `Vec` is a valid, non-error outcome on a GPU-less/headless box) on every implemented
    /// platform
    ///
    #[test]
    fn collect_returns_ok_on_supported_os() {
        assert!(GpuService.collect().is_ok());
    }

    /// `render_does_not_panic()` asserts that rendering GPU info does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = GpuService.collect().unwrap();
        assert!(GpuService.render(&data).is_ok());
    }
}
