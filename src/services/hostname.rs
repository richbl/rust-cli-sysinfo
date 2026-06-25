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
        let hostname =
            read_first_line("/proc/sys/kernel/hostname").unwrap_or_else(|| "unknown".into());
        Ok(HostnameInfo { hostname })
    }

    /// `render()` renders the hostname
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) {
        print_row(label, &data.hostname, &Threshold::None, c);
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    #[test]
    /// `collect_returns_ok_with_non_empty_hostname()` asserts that hostname collection succeeds
    /// and returns a non-empty hostname
    ///
    fn collect_returns_ok_with_non_empty_hostname() {
        let result = HostnameService.collect();
        assert!(result.is_ok(), "collect() must not error on Linux");
        let data = result.unwrap();
        assert!(
            !data.hostname.is_empty(),
            "hostname must not be empty on a running Linux system"
        );
    }

    #[test]
    /// `render_does_not_panic()` asserts that rendering hostname does not panic
    ///
    fn render_does_not_panic() {
        let data = HostnameService.collect().unwrap();

        // render() writes to stdout; we just verify it does not panic
        HostnameService.render("  Hostname:", &data, &Colors::new(false));
    }
}
