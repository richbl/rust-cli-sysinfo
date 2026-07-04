use super::prelude::*;
use crate::core::utils::read_first_line;

/// `HostnameInfo` contains the system hostname read from `/proc/sys/kernel/hostname`
pub struct HostnameInfo {
    pub hostname: String,
}

/// `HostnameService` is a struct for collecting and rendering the system hostname
pub struct HostnameService;

/// `HostnameService` implements the `Service` trait
impl Service for HostnameService {
    type Data = HostnameInfo;

    /// `collect()` reads the hostname from `/proc/sys/kernel/hostname`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let hostname = read_first_line("/proc/sys/kernel/hostname")?;
        Ok(HostnameInfo { hostname })
    }

    /// `render()` renders the hostname
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        print_row(label, &data.hostname, &Threshold::None, c);
        Ok(())
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
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    /// `collect_returns_ok_with_non_empty_hostname()` asserts that hostname collection succeeds
    /// and returns a non-empty hostname
    ///
    #[test]
    fn collect_returns_ok_with_non_empty_hostname() {
        let result = HostnameService.collect();
        assert!(result.is_ok(), "collect() must not error on Linux");
        let data = result.unwrap();
        assert!(
            !data.hostname.is_empty(),
            "hostname must not be empty on a running Linux system"
        );
    }

    /// `render_does_not_panic()` asserts that rendering the hostname does not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = HostnameService.collect().unwrap();

        // render() writes to stdout; we just verify it does not panic
        HostnameService
            .render("Hostname", &data, &Colors::new(false))
            .unwrap();
    }
}
