use super::prelude::*;
use crate::core::utils::find_key_value;

/// `CpuModelInfo` parsed from `/proc/cpuinfo`
#[derive(Default)]
pub struct CpuModelInfo {
    pub model: Option<String>,
}

/// `CpuModelService` is a service for collecting and rendering the CPU model name
pub struct CpuModelService;

/// `CpuModelService` implements the `Service` trait
impl Service for CpuModelService {
    type Data = CpuModelInfo;

    /// `collect()` reads the first `model name` entry from `/proc/cpuinfo`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let model = find_key_value("/proc/cpuinfo", "model name", ':')?;
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
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    /// `collect_returns_ok_on_linux()` asserts that CPU model collection succeeds on a Linux
    /// system
    ///
    #[test]
    fn collect_returns_ok_on_linux() {
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
            "model name must be present on a CPU-bearing Linux system"
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
