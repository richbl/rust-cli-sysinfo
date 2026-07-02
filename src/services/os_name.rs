use super::prelude::*;
use crate::core::utils::find_key_value;

/// `OsInfo` contains the OS name parsed from `/etc/os-release`
pub struct OsInfo {
    pub name: String,
}

/// `OsService` is a service for collecting and rendering the OS name
pub struct OsService;

/// `OsService` implements the `Service` trait
impl Service for OsService {
    type Data = OsInfo;

    /// `collect()` reads `PRETTY_NAME` from `/etc/os-release`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let name = find_key_value("/etc/os-release", "PRETTY_NAME", '=')?.map_or_else(
            || "Unknown Linux".into(),
            |val| val.trim_matches('"').to_string(),
        );

        Ok(OsInfo { name })
    }

    /// `render()` renders the OS name
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        print_row(label, &data.name, &Threshold::None, c);
        Ok(())
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "OS",
            label: "OS",
            description: "Operating system name and version",
            sort_order: 0,
        },
        Box::new(OsService),
    )
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    /// `collect_returns_ok_with_non_empty_name()` asserts that OS name collection succeeds and
    /// returns a non-empty name
    ///
    #[test]
    fn collect_returns_ok_with_non_empty_name() {
        let result = OsService.collect();
        assert!(result.is_ok());
        assert!(!result.unwrap().name.is_empty());
    }

    /// `render_does_not_panic()` asserts that rendering OS name will not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = OsService.collect().unwrap();
        OsService.render("OS", &data, &Colors::new(false)).unwrap();
    }
}
