use std::io;
use thiserror::Error;

/// `AppError` is the application-level error type returned by all service `collect()` calls
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Data unavailable: {0}")]
    DataUnavailable(String),
}

impl From<utwt::ParseError> for AppError {
    /// `from()` converts `utwt::ParseError` into `AppError` to facilitate error handling
    ///
    fn from(e: utwt::ParseError) -> Self {
        match e {
            utwt::ParseError::Io(io_err) => Self::Io(io_err),
            utwt::ParseError::Utmp(utmp_err) => Self::DataUnavailable(utmp_err.to_string()),
            _ => Self::DataUnavailable("unknown utmp parse error".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    // AppError::IO tests

    /// `display_io_wraps_underlying_message()` asserts that the `Io` variant of `AppError` wraps
    /// the underlying I/O error message
    ///
    #[test]
    fn display_io_wraps_underlying_message() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::Io(io_err);
        assert!(app_err.to_string().starts_with("IO error:"));
        assert!(app_err.to_string().contains("file not found"));
    }

    // AppError::DataUnavailable test

    /// `display_data_unavailable_includes_message()` asserts that the `DataUnavailable` variant of
    /// `AppError` includes the underlying message
    ///
    #[test]
    fn display_data_unavailable_includes_message() {
        let err = AppError::DataUnavailable("sensor offline".to_string());
        assert_eq!(err.to_string(), "Data unavailable: sensor offline");
    }

    // From<io::Error> test

    /// `from_io_error_produces_io_variant()` asserts that `AppError::from()` converts an
    /// `io::Error` into an `AppError::Io`
    ///
    #[test]
    fn from_io_error_produces_io_variant() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    /// `from_io_error_preserves_message()` asserts that the `io::Error` message is preserved
    ///
    #[test]
    fn from_io_error_preserves_message() {
        let io_err = io::Error::new(io::ErrorKind::TimedOut, "connection timed out");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("connection timed out"));
    }

    // std::error::Error test

    /// `app_error_is_boxable_as_std_error()` asserts that `AppError` implements
    /// `std::error::Error`
    ///
    #[test]
    fn app_error_is_boxable_as_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(AppError::DataUnavailable("test".to_string()));
        assert!(!err.to_string().is_empty());
    }

    /// `io_variant_exposes_source()` asserts that the `Io` variant of `AppError` exposes its cause
    /// via `source()`
    ///
    #[test]
    fn io_variant_exposes_source() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        let app_err = AppError::Io(io_err);
        assert!(
            app_err.source().is_some(),
            "AppError::Io must expose its cause via source()"
        );
    }
}
