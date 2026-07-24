#[cfg(target_os = "linux")]
use crate::core::error::AppError;
#[cfg(target_os = "linux")]
use std::{fs, path::Path};

use crate::constants::APP_NAME;

/// `read_hex_u16()` reads a hexadecimal value from a file and returns it as a u16
///
#[cfg(target_os = "linux")]
pub fn read_hex_u16(path: &Path) -> Result<u16, AppError> {
    let contents = fs::read_to_string(path)?;
    let value = contents.trim().trim_start_matches("0x");

    u16::from_str_radix(value, 16)
        .map_err(|_| AppError::DataUnavailable(format!("invalid hex u16 in {}", path.display())))
}

/// `generate_title()` creates a single-line titlebar of exactly `sep_len` display columns
///
/// The title is formatted as `APP_NAME` followed by enough `─` characters to fill the
/// remaining width, so the total rendered length equals `sep_len`. Callers control the
/// width:
/// - Static contexts (help, services list): pass `SEP_FALLBACK.chars().count()`
/// - Dynamic contexts (services output): pass the computed content column width
///
pub fn generate_title(sep_len: usize) -> String {
    let app_len = APP_NAME.chars().count() + 1; // +1 for the separating space

    // Prevent underflow when APP_NAME is longer than (or equal to) sep_len
    let fill = sep_len.saturating_sub(app_len);

    let suffix: String = "─".repeat(fill);

    format!("{APP_NAME} {suffix}")
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    /// `temp_file_with()` helper creates a named temporary file pre-filled with `content`,
    /// returning a `NamedTempFile` that deletes itself on drop (RAII)
    ///
    fn temp_file_with(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("create temp file");
        f.write_all(content.as_bytes()).expect("write temp file");
        f
    }

    /// `read_hex_u16_with_0x_prefix()` reads a hex value from a file and asserts that it is read
    /// with the 0x prefix
    ///
    #[test]
    fn read_hex_u16_with_0x_prefix() {
        let f = temp_file_with("0x8086\n");
        assert_eq!(read_hex_u16(f.path()).unwrap(), 0x8086_u16);
    }

    /// `read_hex_u16_without_0x_prefix()` reads a hex value from a file and asserts that it is
    /// parsed without the 0x prefix
    ///
    #[test]
    fn read_hex_u16_without_prefix() {
        let f = temp_file_with("8086\n");
        assert_eq!(read_hex_u16(f.path()).unwrap(), 0x8086_u16);
    }

    /// `read_hex_u16_max_value()` reads a hex value from a file and asserts that it is `u16::MAX`
    ///
    #[test]
    fn read_hex_u16_max_value() {
        let f = temp_file_with("0xFFFF\n");
        assert_eq!(read_hex_u16(f.path()).unwrap(), u16::MAX);
    }

    /// `read_hex_u16_zero_value()` reads a hex value from a file and asserts that it is 0
    ///
    #[test]
    fn read_hex_u16_zero_value() {
        let f = temp_file_with("0x0000\n");
        assert_eq!(read_hex_u16(f.path()).unwrap(), 0_u16);
    }

    /// `read_hex_u16_overflow_u16_returns_err()` ensures that the function returns `Err` if the
    /// value exceeds `u16::MAX`
    ///
    #[test]
    fn read_hex_u16_overflow_u16_returns_err() {
        let f = temp_file_with("0x10000\n");
        assert!(read_hex_u16(f.path()).is_err());
    }

    /// `read_hex_u16_non_hex_content_returns_err()` ensures that the function returns `Err` if
    /// the file contains non-hex content
    ///
    #[test]
    fn read_hex_u16_non_hex_content_returns_err() {
        let f = temp_file_with("not_hex\n");
        assert!(read_hex_u16(f.path()).is_err());
    }

    /// `read_hex_u16_empty_file_returns_err()` ensures that the function returns `Err` if the
    /// file is empty
    ///
    #[test]
    fn read_hex_u16_empty_file_returns_err() {
        let f = temp_file_with("");
        assert!(read_hex_u16(f.path()).is_err());
    }

    /// `read_hex_u16_missing_file_returns_err()` ensures that the function returns `Err` if the
    /// file is missing
    ///
    #[test]
    fn read_hex_u16_missing_file_returns_err() {
        let path = std::path::Path::new("/nonexistent/rcs_missing_hex_for_test");
        assert!(read_hex_u16(path).is_err());
    }

    /// `read_hex_u16_trims_surrounding_whitespace()` ensures that the function trims leading and
    /// trailing whitespace
    ///
    #[test]
    fn read_hex_u16_trims_surrounding_whitespace() {
        let f = temp_file_with("  0x10DE  \n");
        assert_eq!(read_hex_u16(f.path()).unwrap(), 0x10DE_u16);
    }
}
