use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

/// `read_first_line()` reads the first line of a file at the given path
///
pub fn read_first_line(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempFile(std::path::PathBuf);

    impl TempFile {
        /// `write()` creates a temporary file with the given content and returns its path,
        /// following the RAII pattern
        ///
        fn write(content: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("rcs_test_{}_{n}", std::process::id()));
            let mut f = std::fs::File::create(&path).expect("create temp file");
            f.write_all(content.as_bytes()).expect("write temp file");
            Self(path)
        }

        /// `path_str()` returns the path of the temporary file
        ///
        fn path_str(&self) -> &str {
            self.0.to_str().expect("temp path is valid UTF-8")
        }
    }

    impl Drop for TempFile {
        /// `drop()` deletes the temporary file
        ///
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    /// `read_first_line()` asserts that only the first line of a file is returned
    ///
    fn read_first_line_returns_only_first_line() {
        let f = TempFile::write("first\nsecond\nthird\n");
        assert_eq!(read_first_line(f.path_str()), Some("first".to_string()));
    }

    #[test]
    /// `read_first_line_strips_trailing_newline()` asserts that newlines are stripped
    ///
    fn read_first_line_strips_trailing_newline() {
        let f = TempFile::write("no_newline_leaked\n");
        let result = read_first_line(f.path_str()).unwrap();
        assert!(!result.ends_with('\n'), "newline must be stripped");
        assert_eq!(result, "no_newline_leaked");
    }

    #[test]
    /// `read_first_line_no_trailing_newline_in_file()` asserts that the function returns
    /// `Some()` even if the file has no trailing newline
    ///
    fn read_first_line_no_trailing_newline_in_file() {
        let f = TempFile::write("bare");
        assert_eq!(read_first_line(f.path_str()), Some("bare".to_string()));
    }

    #[test]
    /// `read_first_line_empty_file_returns_some_empty_string()` asserts that the function returns
    /// `Some("")` even if the file is empty
    ///
    fn read_first_line_empty_file_returns_some_empty_string() {
        // An empty file has no content but open() succeeds and read_line() returns Ok(0), so the
        // function returns Some("") rather than None
        let f = TempFile::write("");
        assert_eq!(read_first_line(f.path_str()), Some(String::new()));
    }

    #[test]
    /// `read_first_line_missing_file_returns_none()` asserts that the function returns `None` if the
    /// file is missing
    ///
    fn read_first_line_missing_file_returns_none() {
        assert_eq!(
            read_first_line("/nonexistent/rcs_missing_file_for_test"),
            None
        );
    }

    #[test]
    /// `read_first_line_preserves_interior_whitespace()` asserts that the function preserves
    /// whitespace in the file
    ///
    fn read_first_line_preserves_interior_whitespace() {
        let f = TempFile::write("  two  spaces  \n");
        assert_eq!(
            read_first_line(f.path_str()),
            Some("  two  spaces  ".to_string())
        );
    }

    #[test]
    /// `read_hex_u16()` reads a hex value from a file and asserts that it is read
    /// with the 0x prefix
    ///
    fn read_hex_u16_with_0x_prefix() {
        let f = TempFile::write("0x8086\n");
        assert_eq!(read_hex_u16(f.0.as_path()), Some(0x8086_u16));
    }

    #[test]
    /// `read_hex_u16_without_0x_prefix()` reads a hex value from a file and asserts that it is
    /// parsed without the 0x prefix
    ///
    fn read_hex_u16_without_prefix() {
        let f = TempFile::write("8086\n");
        assert_eq!(read_hex_u16(f.0.as_path()), Some(0x8086_u16));
    }

    #[test]
    /// `read_hex_u16_max_value()` reads a hex value from a file and asserts that it is `u16::MAX`
    ///
    fn read_hex_u16_max_value() {
        let f = TempFile::write("0xFFFF\n");
        assert_eq!(read_hex_u16(f.0.as_path()), Some(u16::MAX));
    }

    #[test]
    /// `read_hex_u16_zero_value()` reads a hex value from a file and asserts that it is 0
    ///
    fn read_hex_u16_zero_value() {
        let f = TempFile::write("0x0000\n");
        assert_eq!(read_hex_u16(f.0.as_path()), Some(0_u16));
    }

    #[test]
    /// `read_hex_u16_overflow_u16_returns_none()` asserts that the function returns `None` if the
    /// value exceeds `u16::MAX`
    ///
    fn read_hex_u16_overflow_u16_returns_none() {
        // 0x10000 exceeds u16::MAX — must not wrap or panic
        let f = TempFile::write("0x10000\n");
        assert_eq!(read_hex_u16(f.0.as_path()), None);
    }

    #[test]
    /// `read_hex_u16_non_hex_content_returns_none()` asserts that the function returns `None` if
    /// the file contains non-hex content
    ///
    fn read_hex_u16_non_hex_content_returns_none() {
        let f = TempFile::write("not_hex\n");
        assert_eq!(read_hex_u16(f.0.as_path()), None);
    }

    #[test]
    /// `read_hex_u16_empty_file_returns_none()` asserts that the function returns `None` if the
    /// file is empty
    ///
    fn read_hex_u16_empty_file_returns_none() {
        let f = TempFile::write("");
        assert_eq!(read_hex_u16(f.0.as_path()), None);
    }

    /// `read_hex_u16_missing_file_returns_none()` asserts that the function returns `None` if the
    /// file is missing
    ///
    #[test]
    fn read_hex_u16_missing_file_returns_none() {
        let path = std::path::Path::new("/nonexistent/rcs_missing_hex_for_test");
        assert_eq!(read_hex_u16(path), None);
    }

    #[test]
    /// `read_hex_u16_trims_surrounding_whitespace()` asserts that the function trims leading and
    /// trailing whitespace
    ///
    fn read_hex_u16_trims_surrounding_whitespace() {
        // Verify that leading/trailing whitespace (common in sysfs files) is handled
        let f = TempFile::write("  0x10DE  \n");
        assert_eq!(read_hex_u16(f.0.as_path()), Some(0x10DE_u16));
    }
}
