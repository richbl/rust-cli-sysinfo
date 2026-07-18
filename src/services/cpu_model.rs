use super::prelude::*;

/// `CpuModelInfo` contains the CPU model/brand string
#[derive(Default)]
pub struct CpuModelInfo {
    pub model: Option<String>,
}

/// `CpuModelService` is a service for collecting and rendering the CPU model name
pub struct CpuModelService;

/// `CpuModelService` implements the `Service` trait
impl Service for CpuModelService {
    type Data = CpuModelInfo;

    /// `collect()` creates a fresh, minimal `sysinfo::System`, refreshes CPU info, and reads the
    /// brand string off the first logical CPU
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let mut sys = sysinfo::System::new();
        sys.refresh_cpu_all();

        let model = sys.cpus().first().map(|cpu| cpu.brand().to_string());
        Ok(CpuModelInfo { model })
    }

    /// `render()` renders the CPU model name
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.model.clone().unwrap_or_else(|| "Unknown".to_string()),
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
            token: "CPU",
            label: "CPU",
            description: "CPU model",
            sort_order: 10,
        },
        Box::new(CpuModelService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

    /// `collect_returns_ok_on_supported_os()` asserts that CPU model collection succeeds on a
    /// supported system
    ///
    #[test]
    fn collect_returns_ok_on_supported_os() {
        let result = CpuModelService.collect();
        assert!(result.is_ok());
    }

    /// `model_name_is_some_and_non_empty()` asserts that CPU model name is retrieved and is
    /// non-empty
    ///
    #[test]
    fn model_name_is_some_and_non_empty() {
        let data = CpuModelService.collect().unwrap();
        assert!(
            data.model.is_some(),
            "model name must be present on a CPU-driven system"
        );
        assert!(!data.model.unwrap().is_empty());
    }

    /// `render_does_not_panic()` asserts that rendering CPU model info does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = CpuModelService.collect().unwrap();
        CpuModelService.render(&data).unwrap();
    }
}
