use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use pci_ids::{Device, FromId, Vendor};

use super::prelude::*;
use crate::core::error::AppError;
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
            models: collect_gpu_models()?,
        })
    }

    /// `render()` renders GPU model name(s) as a single row, newline-separated for multiple GPUs
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) {
        let separator = format!("\n{:width$}", "", width = label.len() + 1);
        let gpu_str = if data.models.is_empty() {
            "Unknown".to_string()
        } else {
            data.models.join(&separator)
        };
        print_row(label, &gpu_str, &Threshold::None, c);
    }
}

/// `collect_gpu_models()` discovers GPU display names by scanning `/sys/class/drm` card entries
///
fn collect_gpu_models() -> Result<Vec<String>, AppError> {
    let mut seen_paths: HashSet<PathBuf> = HashSet::new();
    let mut names = Vec::new();

    let entries = fs::read_dir("/sys/class/drm").map_err(AppError::Io)?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(card_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Process only top-level card entries (e.g., "card0"), not connectors (e.g., "card0-DP-1")
        if !card_name.starts_with("card") || card_name.contains('-') {
            continue;
        }

        let device_path = path.join("device");
        let Ok(canonical_path) = fs::canonicalize(&device_path) else {
            continue; // Skip if path cannot be resolved
        };

        // Deduplicate by canonical path
        if seen_paths.insert(canonical_path)
            && let Some(name) = gpu_name_from_card(&path)?
        {
            names.push(name);
        }
    }

    names.sort();
    Ok(names)
}

/// `gpu_name_from_card()` resolves a GPU display name from a DRM card path via PCI vendor/device
/// IDs
/// Returns `Ok(None)` if the card does not expose valid PCI IDs (e.g. virtual cards)
///
fn gpu_name_from_card(card_path: &Path) -> Result<Option<String>, AppError> {
    let device_path = card_path.join("device");

    // Read vendor ID, skipping the card gracefully if the file is missing/inaccessible
    let vendor_id = match read_hex_u16(&device_path.join("vendor")) {
        Ok(id) => id,
        Err(AppError::Io(_)) => return Ok(None),
        Err(e) => return Err(e),
    };

    // Read device ID, skipping the card gracefully if the file is missing/inaccessible
    let device_id = match read_hex_u16(&device_path.join("device")) {
        Ok(id) => id,
        Err(AppError::Io(_)) => return Ok(None),
        Err(e) => return Err(e),
    };

    let Some(vendor) = Vendor::from_id(vendor_id) else {
        return Ok(None);
    };

    let name = Device::from_vid_pid(vendor_id, device_id).map_or_else(
        || vendor.name().to_owned(),
        |device| format!("{} {}", vendor.name(), device.name()),
    );

    Ok(Some(name))
}
