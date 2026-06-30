use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use crate::core::error::AppError;

/// `read_first_line()` reads the first line of a file at the given path
///
pub fn read_first_line(path: impl AsRef<std::path::Path>) -> Result<String, AppError> {
    let file = fs::File::open(path.as_ref())?;
    let mut line = String::new();

    io::BufReader::new(file).read_line(&mut line)?;

    let len = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(len);

    Ok(line)
}

/// `read_hex_u16()` reads a hexadecimal value from a file and returns it as a u16
///
pub fn read_hex_u16(path: &Path) -> Result<u16, AppError> {
    let contents = fs::read_to_string(path)?;
    let value = contents.trim().trim_start_matches("0x");

    u16::from_str_radix(value, 16)
        .map_err(|_| AppError::DataUnavailable(format!("invalid hex u16 in {}", path.display())))
}

/// `find_key_value` searches a file line-by-line for a key starting with a prefix, splitting the
/// line by the provided delimiter and returning the trimmed value
///
pub fn find_key_value(
    path: impl AsRef<std::path::Path>,
    key_prefix: &str,
    delimiter: char,
) -> Result<Option<String>, AppError> {
    let file = std::fs::File::open(path)?;

    for line in std::io::BufReader::new(file).lines() {
        let line = line?;
        if line.starts_with(key_prefix)
            && let Some((_, val)) = line.split_once(delimiter)
        {
            return Ok(Some(val.trim().to_string()));
        }
    }

    Ok(None)
}

#[cfg(test)]
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

    /// `read_first_line_returns_only_first_line()` asserts that only the first line of a file is
    /// returned
    ///
    #[test]
    fn read_first_line_returns_only_first_line() {
        let f = temp_file_with("first\nsecond\nthird\n");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")).unwrap(),
            "first".to_string()
        );
    }

    /// `read_first_line_strips_trailing_newline()` ensures that newlines are stripped
    ///
    #[test]
    fn read_first_line_strips_trailing_newline() {
        let f = temp_file_with("no_newline_leaked\n");
        let result = read_first_line(f.path().to_str().expect("temp path is valid UTF-8")).unwrap();
        assert!(!result.ends_with('\n'), "newline must be stripped");
        assert_eq!(result, "no_newline_leaked");
    }

    /// `read_first_line_no_trailing_newline_in_file()` ensures that the function returns
    /// `Ok()` even if the file has no trailing newline
    ///
    #[test]
    fn read_first_line_no_trailing_newline_in_file() {
        let f = temp_file_with("bare");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")).unwrap(),
            "bare".to_string()
        );
    }

    /// `read_first_line_empty_file_returns_ok_empty_string()` ensures that the function returns
    /// `Ok("")` even if the file is empty
    ///
    #[test]
    fn read_first_line_empty_file_returns_ok_empty_string() {
        let f = temp_file_with("");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")).unwrap(),
            String::new()
        );
    }

    /// `read_first_line_missing_file_returns_err()` ensures that the function returns `Err` if the
    /// file is missing
    ///
    #[test]
    fn read_first_line_missing_file_returns_err() {
        assert!(read_first_line("/nonexistent/rcs_missing_file_for_test").is_err());
    }

    /// `read_first_line_preserves_interior_whitespace()` ensures that the function preserves
    /// whitespace in the file
    ///
    #[test]
    fn read_first_line_preserves_interior_whitespace() {
        let f = temp_file_with("  two  spaces  \n");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")).unwrap(),
            "  two  spaces  ".to_string()
        );
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
