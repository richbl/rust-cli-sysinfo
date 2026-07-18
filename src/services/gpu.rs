#[cfg(target_os = "linux")]
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[cfg(not(target_os = "linux"))]
use crate::constants::NOT_YET_IMPLEMENTED;

use super::prelude::*;
use crate::constants::{INDENT, LABEL_WIDTH};
use crate::core::error::AppError;

/// Deduplicated list of GPU display names
///
pub struct GpuInfo {
    pub models: Vec<String>,
}

/// Service for collecting and rendering GPU model name(s)
pub struct GpuService;

impl Service for GpuService {
    type Data = GpuInfo;

    /// `collect()` delegates to the platform-specific implementation below
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(GpuInfo {
            models: collect_gpu_models()?,
        })
    }

    /// `render()` renders GPU model name(s)
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        let separator = format!("\n{:width$}", "", width = INDENT.len() + LABEL_WIDTH + 1);
        let value = if data.models.is_empty() {
            "Unknown".to_string()
        } else {
            data.models.join(&separator)
        };
        Ok(RenderedRow {
            value,
            threshold: Threshold::None,
        })
    }
}

/// `collect_gpu_models()` discovers GPU display names by scanning `/sys/class/drm` card entries
/// and resolving vendor/device PCI IDs into display names
///
#[cfg(target_os = "linux")]
fn collect_gpu_models() -> Result<Vec<String>, AppError> {
    let mut seen_paths: HashSet<PathBuf> = HashSet::new();
    let mut names = Vec::new();

    let entries = fs::read_dir("/sys/class/drm")?;

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
///
#[cfg(target_os = "linux")]
fn gpu_name_from_card(card_path: &Path) -> Result<Option<String>, AppError> {
    use crate::core::utils::read_hex_u16;
    use pci_ids::{Device, FromId, Vendor};

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

/// `collect_gpu_models()` — non-Linux fallback
///
/// `sysinfo`'s native GPU support (Linux/macOS/Windows) isn't in a published release yet,
/// so rather than leaving Windows/macOS silently rendering, this surfaces as an explicit
/// collection error
///
#[cfg(not(target_os = "linux"))]
fn collect_gpu_models() -> Result<Vec<String>, AppError> {
    Err(AppError::DataUnavailable(NOT_YET_IMPLEMENTED.into()))
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "GPU",
            label: "GPU(s)",
            description: "GPU model(s)",
            sort_order: 20,
        },
        Box::new(GpuService),
    )
}
