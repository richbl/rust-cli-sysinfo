use std::fmt;
use std::io;

/// `AppError` is the application-level error type returned by all service `collect()` calls
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    DataUnavailable(String),
}

/// `AppError` implements the standard `Display` trait
impl fmt::Display for AppError {
    /// `fmt()` formats `AppError` variants for end-user display
    ///
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // AppError::IO tests

    #[test]
    /// `display_io_wraps_underlying_message()` asserts that the `Io` variant of `AppError` wraps
    /// the underlying I/O error message
    ///
    fn display_io_wraps_underlying_message() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::Io(io_err);
        assert!(app_err.to_string().starts_with("IO error:"));
        assert!(app_err.to_string().contains("file not found"));
    }

    // AppError::DataUnavailable test

    #[test]
    /// `display_data_unavailable_includes_message()` asserts that the `DataUnavailable` variant of
    /// `AppError` includes the underlying message
    ///
    fn display_data_unavailable_includes_message() {
        let err = AppError::DataUnavailable("sensor offline".to_string());
        assert_eq!(err.to_string(), "Data unavailable: sensor offline");
    }

    // From<io::Error> test

    #[test]
    /// `from_io_error_produces_io_variant()` asserts that `AppError::from()` converts an
    /// `io::Error` into an `AppError::Io`
    ///
    fn from_io_error_produces_io_variant() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    /// `from_io_error_preserves_message()` asserts that the `io::Error` message is preserved
    ///
    fn from_io_error_preserves_message() {
        let io_err = io::Error::new(io::ErrorKind::TimedOut, "connection timed out");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("connection timed out"));
    }

    // std::error::Error test

    #[test]
    /// `app_error_is_boxable_as_std_error()` asserts that `AppError` implements
    /// `std::error::Error`
    ///
    fn app_error_is_boxable_as_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(AppError::DataUnavailable("test".to_string()));
        assert!(!err.to_string().is_empty());
    }
}
