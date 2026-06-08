use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use pci_ids::{Device, FromId, Vendor};

use super::Service;
use crate::core::error::AppError;
use crate::core::utils::{read_first_line, read_hex_u16};
use crate::presentation::colors::{Colors, Threshold};
use crate::presentation::format::{format_uptime, print_row};

/// System-level information collected from `/proc`, `/sys`, and `/etc`
pub struct SystemInfo {
    pub hostname: String,
    pub cpu_model: Option<String>,
    /// Deduplicated list of GPU display names
    pub gpu_model: Vec<String>,
    pub os: String,
    pub kernel: String,
    pub uptime_secs: Option<u64>,
}

/// Service for collecting and rendering system-level metrics
pub struct SystemService;

impl Service for SystemService {
    type Data = SystemInfo;

    /// `collect()` collects hostname, OS name, kernel version, uptime, CPU model, and GPU model(s)
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let hostname =
            read_first_line("/proc/sys/kernel/hostname").unwrap_or_else(|| "unknown".into());
        let kernel =
            read_first_line("/proc/sys/kernel/osrelease").unwrap_or_else(|| "unknown".into());

        let os = read_os_name().unwrap_or_else(|| "Unknown Linux".into());

        let uptime_secs = read_first_line("/proc/uptime").and_then(|raw| {
            raw.split_whitespace()
                .next()?
                .split('.')
                .next()?
                .parse()
                .ok()
        });

        let cpu_model = collect_cpu_model();
        let gpu_model = collect_gpu_model();

        Ok(SystemInfo {
            hostname,
            cpu_model,
            gpu_model,
            os,
            kernel,
            uptime_secs,
        })
    }

    /// `render()` renders hostname, CPU, GPU(s), kernel version, and system uptime
    ///
    fn render(&self, sys: &Self::Data, c: &Colors) {
        print_row("  Hostname:", &sys.hostname, &Threshold::None, c);

        print_row(
            "  CPU:",
            sys.cpu_model.as_deref().unwrap_or("Unknown"),
            &Threshold::None,
            c,
        );

        let gpu_str = if sys.gpu_model.is_empty() {
            "Unknown".to_string()
        } else {
            sys.gpu_model.join("\n                 ")
        };

        print_row("  GPU(s):", &gpu_str, &Threshold::None, c);
        print_row("  Kernel:", &sys.kernel, &Threshold::None, c);

        let uptime_str = sys
            .uptime_secs
            .map_or_else(|| "unknown".to_string(), format_uptime);

        print_row("  Uptime:", &uptime_str, &Threshold::None, c);
    }
}

/// `read_os_name()` reads the `PRETTY_NAME` field from `/etc/os-release`
///
fn read_os_name() -> Option<String> {
    let file = File::open("/etc/os-release").ok()?;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
            return Some(val.trim_matches('"').to_string());
        }
    }

    None
}

/// `collect_cpu_model()` reads the CPU model name from the first `model name` entry in
/// `/proc/cpuinfo`
///
fn collect_cpu_model() -> Option<String> {
    let file = File::open("/proc/cpuinfo").ok()?;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("model name")
            && let Some((_, val)) = line.split_once(':')
        {
            return Some(val.trim().to_string());
        }
    }

    None
}

/// `collect_gpu_model()` discovers GPU display names by scanning `/sys/class/drm` and resolving
/// vendor/device PCI IDs
///
fn collect_gpu_model() -> Vec<String> {
    let mut names = BTreeSet::new();

    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };

    // Collect unique GPU display names
    for entry in entries.flatten() {
        let path = entry.path();

        let Some(card_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Process only top-level card entries (e.g., "card0"), not connectors (e.g., "card0-DP-1")
        if !card_name.starts_with("card") || card_name.contains('-') {
            continue;
        }

        if let Some(name) = gpu_name_from_card(&path) {
            names.insert(name);
        }
    }

    names.into_iter().collect()
}

/// `gpu_name_from_card()` resolves a GPU display name from a DRM card path via PCI
/// vendor/device IDs
///
fn gpu_name_from_card(card_path: &Path) -> Option<String> {
    let device_path = card_path.join("device");

    let vendor_id = read_hex_u16(&device_path.join("vendor"))?;
    let device_id = read_hex_u16(&device_path.join("device"))?;

    let vendor = Vendor::from_id(vendor_id)?;

    // Use full "Vendor Device" name when known; fall back to vendor name only
    Device::from_vid_pid(vendor_id, device_id)
        .map(|device| format!("{} {}", vendor.name(), device.name()))
        .or_else(|| Some(vendor.name().to_owned()))
}
