use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// `read_first_line()` reads the first line of a file at the given path
///
pub fn read_first_line(path: impl AsRef<std::path::Path>) -> Option<String> {
    let file = fs::File::open(path.as_ref()).ok()?;
    let mut line = String::new();

    io::BufReader::new(file).read_line(&mut line).ok()?;

    let len = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(len);

    Some(line)
}

/// `read_hex_u16()` reads a hexadecimal value from a file and returns it as a u16
///
pub fn read_hex_u16(path: &Path) -> Option<u16> {
    let contents = fs::read_to_string(path).ok()?;
    let value = contents.trim().trim_start_matches("0x");

    u16::from_str_radix(value, 16).ok()
}

/// `c_char_array_to_string()` safely converts a fixed-size C character array (null-terminated)
/// from a `libc` struct into a Rust `String`.
///
pub fn c_char_array_to_string(c_array: &[libc::c_char]) -> String {
    c_array
        .iter()
        .map(|&c| c.cast_unsigned())
        .take_while(|&c| c != 0)
        .map(|c| c as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    /// `temp_file_with()` creates a named temporary file pre-filled with `content`, returning a
    /// `NamedTempFile` that deletes itself on drop (RAII)
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
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")),
            Some("first".to_string())
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
    /// `Some()` even if the file has no trailing newline
    ///
    #[test]
    fn read_first_line_no_trailing_newline_in_file() {
        let f = temp_file_with("bare");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")),
            Some("bare".to_string())
        );
    }

    /// `read_first_line_empty_file_returns_some_empty_string()` ensures that the function returns
    /// `Some("")` even if the file is empty
    ///
    #[test]
    fn read_first_line_empty_file_returns_some_empty_string() {
        // An empty file has no content but open() succeeds and read_line() returns Ok(0), so the
        // function returns Some("") rather than None
        let f = temp_file_with("");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")),
            Some(String::new())
        );
    }

    /// `read_first_line_missing_file_returns_none()` ensures that the function returns `None` if the
    /// file is missing
    ///
    #[test]
    fn read_first_line_missing_file_returns_none() {
        assert_eq!(
            read_first_line("/nonexistent/rcs_missing_file_for_test"),
            None
        );
    }

    /// `read_first_line_preserves_interior_whitespace()` ensures that the function preserves
    /// whitespace in the file
    ///
    #[test]
    fn read_first_line_preserves_interior_whitespace() {
        let f = temp_file_with("  two  spaces  \n");
        assert_eq!(
            read_first_line(f.path().to_str().expect("temp path is valid UTF-8")),
            Some("  two  spaces  ".to_string())
        );
    }

    /// `read_hex_u16_with_0x_prefix()` reads a hex value from a file and asserts that it is read
    /// with the 0x prefix
    ///
    #[test]
    fn read_hex_u16_with_0x_prefix() {
        let f = temp_file_with("0x8086\n");
        assert_eq!(read_hex_u16(f.path()), Some(0x8086_u16));
    }

    /// `read_hex_u16_without_0x_prefix()` reads a hex value from a file and asserts that it is
    /// parsed without the 0x prefix
    ///
    #[test]
    fn read_hex_u16_without_prefix() {
        let f = temp_file_with("8086\n");
        assert_eq!(read_hex_u16(f.path()), Some(0x8086_u16));
    }

    /// `read_hex_u16_max_value()` reads a hex value from a file and asserts that it is `u16::MAX`
    ///
    #[test]
    fn read_hex_u16_max_value() {
        let f = temp_file_with("0xFFFF\n");
        assert_eq!(read_hex_u16(f.path()), Some(u16::MAX));
    }

    /// `read_hex_u16_zero_value()` reads a hex value from a file and asserts that it is 0
    ///
    #[test]
    fn read_hex_u16_zero_value() {
        let f = temp_file_with("0x0000\n");
        assert_eq!(read_hex_u16(f.path()), Some(0_u16));
    }

    /// `read_hex_u16_overflow_u16_returns_none()` ensures that the function returns `None` if the
    /// value exceeds `u16::MAX`
    ///
    #[test]
    fn read_hex_u16_overflow_u16_returns_none() {
        // 0x10000 exceeds u16::MAX — must not wrap or panic
        let f = temp_file_with("0x10000\n");
        assert_eq!(read_hex_u16(f.path()), None);
    }

    /// `read_hex_u16_non_hex_content_returns_none()` ensures that the function returns `None` if
    /// the file contains non-hex content
    ///
    #[test]
    fn read_hex_u16_non_hex_content_returns_none() {
        let f = temp_file_with("not_hex\n");
        assert_eq!(read_hex_u16(f.path()), None);
    }

    /// `read_hex_u16_empty_file_returns_none()` ensures that the function returns `None` if the
    /// file is empty
    ///
    #[test]
    fn read_hex_u16_empty_file_returns_none() {
        let f = temp_file_with("");
        assert_eq!(read_hex_u16(f.path()), None);
    }

    /// `read_hex_u16_missing_file_returns_none()` ensures that the function returns `None` if the
    /// file is missing
    ///
    #[test]
    fn read_hex_u16_missing_file_returns_none() {
        let path = std::path::Path::new("/nonexistent/rcs_missing_hex_for_test");
        assert_eq!(read_hex_u16(path), None);
    }

    /// `read_hex_u16_trims_surrounding_whitespace()` ensures that the function trims leading and
    /// trailing whitespace
    ///
    #[test]
    fn read_hex_u16_trims_surrounding_whitespace() {
        // Verify that leading/trailing whitespace (common in sysfs files) is handled
        let f = temp_file_with("  0x10DE  \n");
        assert_eq!(read_hex_u16(f.path()), Some(0x10DE_u16));
    }

    /// `c_char_array_to_string_extracts_null_terminated_string()` asserts that it extracts up to
    /// the null terminator
    ///
    #[test]
    fn c_char_array_to_string_extracts_null_terminated_string() {
        let arr: [libc::c_char; 8] = [116, 101, 115, 116, 0, 0, 0, 0]; // "test\0\0\0\0"
        assert_eq!(c_char_array_to_string(&arr), "test");
    }

    /// `c_char_array_to_string_handles_full_array_no_null()` asserts that it handles an array with
    /// no null terminator
    ///
    #[test]
    fn c_char_array_to_string_handles_full_array_no_null() {
        let arr: [libc::c_char; 4] = [102, 111, 111, 0]; // "foo\0"
        assert_eq!(c_char_array_to_string(&arr), "foo");
    }

    /// `c_char_array_to_string_empty_array_returns_empty_string()` asserts that an array starting
    /// with null returns empty
    ///
    #[test]
    fn c_char_array_to_string_empty_array_returns_empty_string() {
        let arr: [libc::c_char; 4] = [0, 0, 0, 0];
        assert_eq!(c_char_array_to_string(&arr), "");
    }
}
