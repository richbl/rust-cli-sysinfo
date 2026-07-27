//! Windows: enumerates display adapters via DXGI (DirectX Graphics Infrastructure)
//!

use crate::core::error::AppError;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND, IDXGIFactory1,
};

/// `collect_gpu_models()` enumerates DXGI display adapters and returns their description
/// strings, filtering out non-physical (software/remote) adapters
///
pub(super) fn collect_gpu_models() -> Result<Vec<String>, AppError> {
    // SAFETY: `CreateDXGIFactory1` has no preconditions beyond a valid out-pointer, which
    // `windows`-rs supplies internally via its typed `Result` return
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }?;

    let mut names = Vec::new();
    let mut index = 0u32;

    loop {
        // SAFETY: `EnumAdapters1` is a simple, bounds-checked-by-DXGI enumeration call; it
        // returns `DXGI_ERROR_NOT_FOUND` once `index` exceeds the adapter count rather than
        // reading out of bounds
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(e) => {
                return Err(AppError::DataUnavailable(format!(
                    "DXGI adapter enumeration error: {e}"
                )));
            }
        };
        index += 1;

        // SAFETY: `adapter` was just returned successfully above, so it is a valid COM
        // interface pointer for the duration of this call
        let desc = unsafe { adapter.GetDesc1() }.map_err(|e| {
            AppError::DataUnavailable(format!("DXGI adapter description error: {e}"))
        })?;

        // Skip Microsoft's software rasterizer and other non-physical adapters
        if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
            continue;
        }

        // `Description` is a fixed, NUL-padded UTF-16 buffer
        let name = String::from_utf16_lossy(&desc.Description)
            .trim_end_matches('\u{0}')
            .to_string();

        if !name.is_empty() {
            names.push(name);
        }
    }

    names.sort();
    names.dedup();
    Ok(names)
}
