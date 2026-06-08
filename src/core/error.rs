use std::fmt;
use std::io;

/// `AppError` is the application-level error type returned by all service `collect()` calls
#[derive(Debug)]
pub enum AppError {
    /// Wraps a standard I/O error (e.g., file not found, permission denied)
    Io(io::Error),
    #[allow(dead_code)]
    /// Returned when a value cannot be parsed from its raw string representation
    Parse(String),
    #[allow(dead_code)]
    /// Returned when an expected data source is absent or returns no usable content
    DataUnavailable(String),
}

/// `AppError` implements the standard `Display` trait
impl fmt::Display for AppError {
    /// `fmt()` formats `AppError` variants for end-user display
    ///
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::DataUnavailable(s) => write!(f, "Data unavailable: {s}"),
        }
    }
}

/// `AppError` implements the standard `Error` trait
impl std::error::Error for AppError {}

/// `AppError` implements a conversion from `io::Error` to `AppError`
impl From<io::Error> for AppError {
    /// `from()` converts an `io::Error` into an `AppError::Io`
    ///
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
