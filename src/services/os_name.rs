use super::prelude::*;

/// `OsInfo` contains a human-readable OS name/version string
pub struct OsInfo {
    pub name: String,
}

/// `OsService` is a service for collecting and rendering the OS name
pub struct OsService;

/// `OsService` implements the `Service` trait
impl Service for OsService {
    type Data = OsInfo;

    /// `collect()` returns the OS name and version, or "Unknown OS" if it cannot be determined
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let name = sysinfo::System::long_os_version()
            .or_else(|| {
                let name = sysinfo::System::name()?;
                Some(match sysinfo::System::os_version() {
                    Some(version) => format!("{name} {version}"),
                    None => name,
                })
            })
            .unwrap_or_else(|| "Unknown OS".to_string());

        Ok(OsInfo { name })
    }

    /// `render()` renders the OS name
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.name.clone(),
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
            token: "OS",
            label: "OS",
            description: "Operating system name and version",
            sort_order: 30,
        },
        Box::new(OsService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

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
        OsService.render(&data).unwrap();
    }
}
