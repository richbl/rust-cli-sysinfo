use super::prelude::*;

/// `ActiveIpInfo` contains the primary local IP address currently assigned to this host
#[derive(Default)]
pub struct ActiveIpInfo {
    pub address: String,
}

/// `ActiveIpService` collects the active local IP address
pub struct ActiveIpService;

impl Service for ActiveIpService {
    type Data = ActiveIpInfo;

    /// `collect()` resolves the active local IP address
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let address = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .or_else(|_| local_ip_address::local_ipv6().map(|ip| ip.to_string()))
            .map_err(|e| {
                AppError::DataUnavailable(format!("unable to determine active IP address: {e}"))
            })?;

        Ok(ActiveIpInfo { address })
    }

    /// `render()` displays the active IP address
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.address.clone(),
            threshold: Threshold::None,
        })
    }
}

/// `descriptor()` is this service's registration point
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "IP",
            label: "IP Address",
            description: "Active IP address",
            sort_order: 50,
        },
        Box::new(ActiveIpService),
    )
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod tests {
    use super::*;
    use std::net::IpAddr;

    /// `collect_returns_ok()` asserts that the `collect()` method returns a successful result
    ///
    #[test]
    fn collect_returns_ok() {
        let result = ActiveIpService.collect();
        assert!(result.is_ok());
    }

    /// `active_ip_is_valid()` asserts that the active IP address is a valid IP address
    ///
    #[test]
    fn active_ip_is_valid() {
        let data = ActiveIpService.collect().unwrap();

        let ip = data.address;

        assert!(ip.parse::<IpAddr>().is_ok());
    }

    /// `render_does_not_panic()` asserts that the `render()` method won't panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = ActiveIpService.collect().unwrap();

        ActiveIpService.render(&data).unwrap();
    }
}
