use super::prelude::*;

/// `HostnameInfo` contains the system hostname, resolved via `sysinfo`
pub struct HostnameInfo {
    pub hostname: String,
}

/// `HostnameService` is a struct for collecting and rendering the system hostname
pub struct HostnameService;

/// `HostnameService` implements the `Service` trait
impl Service for HostnameService {
    type Data = HostnameInfo;

    /// `collect()` reads the hostname
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let hostname = sysinfo::System::host_name()
            .ok_or_else(|| AppError::DataUnavailable("hostname unavailable".into()))?;
        Ok(HostnameInfo { hostname })
    }

    /// `render()` renders the hostname
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.hostname.clone(),
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
            token: "HST",
            label: "Hostname",
            description: "System hostname",
            sort_order: 60,
        },
        Box::new(HostnameService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod tests {
    use super::*;

    /// `collect_returns_ok_with_non_empty_hostname()` asserts that hostname collection succeeds
    /// and returns a non-empty hostname
    ///
    #[test]
    fn collect_returns_ok_with_non_empty_hostname() {
        let result = HostnameService.collect();
        assert!(result.is_ok(), "collect() must not error on a supported OS");
        let data = result.unwrap();
        assert!(
            !data.hostname.is_empty(),
            "hostname must not be empty on a running system"
        );
    }

    /// `render_does_not_panic()` asserts that rendering the hostname does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = HostnameService.collect().unwrap();

        HostnameService.render(&data).unwrap();
    }
}
