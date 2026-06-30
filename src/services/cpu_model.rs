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
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        print_row(
            label,
            data.model.as_deref().unwrap_or("Unknown"),
            &Threshold::None,
            c,
        );
        Ok(())
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

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
        CpuModelService
            .render("  CPU:", &data, &Colors::new(false))
            .unwrap();
    }
}
