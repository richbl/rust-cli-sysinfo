use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use pci_ids::{Device, FromId, Vendor};

use super::prelude::*;
use crate::core::utils::read_hex_u16;

/// Deduplicated list of GPU display names discovered via `/sys/class/drm`
pub struct GpuInfo {
    pub models: Vec<String>,
}

/// Service for collecting and rendering GPU model name(s)
pub struct GpuService;

impl Service for GpuService {
    type Data = GpuInfo;

    /// `collect()` scans `/sys/class/drm` and resolves vendor/device PCI IDs into display names
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(GpuInfo {
            models: collect_gpu_models(),
        })
    }

    /// `render()` renders GPU model name(s) as a single row, newline-separated for multiple GPUs
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        let gpu_str = if data.models.is_empty() {
            "Unknown".to_string()
        } else {
            data.models.join("\n                 ")
        };

        print_row("  GPU(s):", &gpu_str, &Threshold::None, c);
    }
}

/// `collect_gpu_models()` discovers GPU display names by scanning `/sys/class/drm` card entries
///
fn collect_gpu_models() -> Vec<String> {
    let mut names = BTreeSet::new();

    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };

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

/// `gpu_name_from_card()` resolves a GPU display name from a DRM card path via PCI vendor/device IDs
///
fn gpu_name_from_card(card_path: &Path) -> Option<String> {
    let device_path = card_path.join("device");

    let vendor_id = read_hex_u16(&device_path.join("vendor"))?;
    let device_id = read_hex_u16(&device_path.join("device"))?;

    let vendor = Vendor::from_id(vendor_id)?;

    Device::from_vid_pid(vendor_id, device_id)
        .map(|device| format!("{} {}", vendor.name(), device.name()))
        .or_else(|| Some(vendor.name().to_owned()))
}
