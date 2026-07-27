//! macOS: enumerates Metal devices via `MTLCopyAllDevices`
//!
//! Metal is the only GPU-facing framework guaranteed on every Mac — Intel machines with a discrete
//! and/or integrated GPU, and Apple Silicon machines whose GPU isn't a PCI device
//!

use crate::core::error::AppError;
use objc2_metal::{MTLCopyAllDevices, MTLDevice};

// `MTLCopyAllDevices` is declared alongside `MTLCreateSystemDefaultDevice` and shares the same
// linkage requirement: `objc2-metal` doesn't pull in `CoreGraphics` on its own, so it must be
// linked explicitly here
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {}

/// `collect_gpu_models()` enumerates every Metal-capable GPU in the system and returns its
/// display name
///
#[allow(clippy::unnecessary_wraps)]
pub(super) fn collect_gpu_models() -> Result<Vec<String>, AppError> {
    // `MTLCopyAllDevices` is a safe function: it only enumerates existing system devices and
    // returns ordinary retained Objective-C objects, no `unsafe` block needed
    let devices = MTLCopyAllDevices();

    let mut names: Vec<String> = devices
        .iter()
        .map(|device| device.name().to_string())
        .collect();

    names.sort();
    names.dedup();
    Ok(names)
}
