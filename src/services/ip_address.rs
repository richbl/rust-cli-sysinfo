use super::prelude::*;
use std::net::UdpSocket;

/// `ActiveIpInfo` contains the primary IP address currently selected by the OS
/// routing table for outbound traffic
#[derive(Default)]
pub struct ActiveIpInfo {
    pub address: String,
}

/// Per RFC 5737/3849, the values of`IP4_ADDR` and `IP6_ADDR` are used to determine the active
/// IP address by connecting a UDP socket to a known public address (no packets are sent)
const IP4_ADDR: &str = "1.1.1.1:80";
const IP6_ADDR: &str = "[2606:4700:4700::1111]:80";

/// `ActiveIpService` collects the active local IP address
pub struct ActiveIpService;

/// `active_ip()` attempts to determine the outbound IPv4 or IPv6 address
///
fn active_ip() -> Result<String, AppError> {
    // Try IPv4 first
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0")
        && socket.connect(IP4_ADDR).is_ok()
        && let Ok(addr) = socket.local_addr()
    {
        return Ok(addr.ip().to_string());
    }

    // Fall back to IPv6
    if let Ok(socket) = UdpSocket::bind("[::]:0")
        && socket.connect(IP6_ADDR).is_ok()
        && let Ok(addr) = socket.local_addr()
    {
        return Ok(addr.ip().to_string());
    }

    // Give up...
    Err(AppError::DataUnavailable(
        "Unable to determine active IP address".into(),
    ))
}

impl Service for ActiveIpService {
    type Data = ActiveIpInfo;

    /// `collect()` attempts to determine the active IP address by connecting a UDP socket to a known public address
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(ActiveIpInfo {
            address: active_ip()?,
        })
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
