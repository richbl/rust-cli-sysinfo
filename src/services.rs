pub mod cpu_model;
pub mod cpu_usage;
pub mod disk;
pub mod gpu;
pub mod hostname;
pub mod kernel;
pub mod load_avg;
pub mod memory;
pub mod os_name;
pub mod uptime;
pub mod users;

use std::any::Any;

use crate::core::error::AppError;
use crate::presentation::colors::Colors;

/// `prelude` re-exports the types shared across all service modules, eliminating
/// duplicate `use` statements in each service file
pub mod prelude {
    pub use super::Service;
    pub use crate::core::error::AppError;
    pub use crate::presentation::colors::Colors;
    pub use crate::presentation::format::{Threshold, print_row};
}

/// `Service` is the common interface implemented by every system information service
pub trait Service {
    type Data: Send + Sync;

    /// `collect()` reads raw system data and returns it
    ///
    fn collect(&self) -> Result<Self::Data, AppError>;

    /// `render()` formats and prints `data` to stdout using the provided colors
    ///
    fn render(&self, data: &Self::Data, colors: &Colors);
}

/// `AnyService` is an object-safe, type-erased counterpart to [`Service`]
///
pub trait AnyService: Send + Sync {
    /// `collect_erased()` calls the underlying service's `collect()`, boxing the
    /// successful result as `Box<dyn Any>`
    ///
    fn collect_erased(&self) -> Result<Box<dyn Any + Send>, AppError>;

    /// `render_erased()` downcasts `data` back to the underlying service's `Data` type
    /// and calls `render()`
    ///
    #[must_use = "a type mismatch produces Err; callers must handle the failure case"]
    fn render_erased(&self, data: &(dyn Any + Send), colors: &Colors) -> Result<(), AppError>;
}

/// Blanket implementation: every [`Service`] is automatically an [`AnyService`]
impl<S> AnyService for S
where
    S: Service + Send + Sync,
    S::Data: 'static + Send + Sync,
{
    /// `collect_erased()` boxes the successful result of `collect()`
    ///
    fn collect_erased(&self) -> Result<Box<dyn Any + Send>, AppError> {
        self.collect()
            .map(|data| Box::new(data) as Box<dyn Any + Send>)
    }

    /// `render_erased()` downcasts `data` to `S::Data` and calls `render()`
    ///
    fn render_erased(&self, data: &(dyn Any + Send), colors: &Colors) -> Result<(), AppError> {
        let typed = data.downcast_ref::<S::Data>().ok_or_else(|| {
            AppError::DataUnavailable(format!(
                "render type mismatch: expected {}, got incompatible Any",
                std::any::type_name::<S::Data>()
            ))
        })?;

        self.render(typed, colors);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    // `DoubleService` and `DoubleData` are defined only to exercise the blanket
    // `AnyService` impl without pulling in any real system service
    struct DoubleData;
    struct DoubleService;

    impl Service for DoubleService {
        type Data = DoubleData;

        fn collect(&self) -> Result<Self::Data, AppError> {
            Ok(DoubleData)
        }

        fn render(&self, _data: &Self::Data, _colors: &Colors) {
            // no-op: render output goes to stdout; only the Result matters here
        }
    }

    /// `collect_erased_returns_ok` asserts that `collect_erased` returns `Ok`
    ///
    #[test]
    fn collect_erased_returns_ok() {
        let result = DoubleService.collect_erased();
        assert!(
            result.is_ok(),
            "collect_erased must not fail for a healthy service"
        );
    }

    /// `collect_erased_boxes_correct_type` asserts that `collect_erased` boxes
    /// the correct type
    ///
    #[test]
    fn collect_erased_boxes_correct_type() {
        let boxed = DoubleService.collect_erased().unwrap();
        assert!(
            boxed.downcast_ref::<DoubleData>().is_some(),
            "boxed value must downcast to DoubleData"
        );
    }

    /// `collect_erased_does_not_downcast_to_wrong_type` asserts that
    /// `collect_erased` does not downcast to the wrong type
    ///
    #[test]
    fn collect_erased_does_not_downcast_to_wrong_type() {
        let boxed = DoubleService.collect_erased().unwrap();
        assert!(
            boxed.downcast_ref::<u32>().is_none(),
            "boxed DoubleData must not downcast to u32"
        );
    }

    // render_erased: happy path test

    /// `render_erased_correct_type_returns_ok` asserts that `render_erased` returns
    /// `Ok` when passed the correct type
    ///
    #[test]
    fn render_erased_correct_type_returns_ok() {
        let data: Box<dyn Any + Send> = Box::new(DoubleData);
        let result = DoubleService.render_erased(data.as_ref(), &Colors::new(false));
        assert!(result.is_ok(), "render with the correct type must succeed");
    }

    /// `round_trip_collect_then_render_erased_succeeds` asserts that
    /// `render_erased` succeeds when passed the result of `collect_erased`
    ///
    #[test]
    fn round_trip_collect_then_render_erased_succeeds() {
        // Simulates the normal collect_services → render_services pipeline
        let collected = DoubleService
            .collect_erased()
            .expect("collect must not fail");
        let result = DoubleService.render_erased(collected.as_ref(), &Colors::new(false));
        assert!(
            result.is_ok(),
            "a correct collect→render round-trip must always succeed"
        );
    }

    // render_erased: type mismatch test

    /// `render_erased_wrong_type_returns_err_not_panics` asserts that `render_erased`
    /// returns `Err` when passed the wrong type
    ///
    #[test]
    fn render_erased_wrong_type_returns_err_not_panics() {
        let wrong: Box<dyn Any + Send> = Box::new("this is not DoubleData");
        let result = DoubleService.render_erased(wrong.as_ref(), &Colors::new(false));
        assert!(
            result.is_err(),
            "a type mismatch must produce Err, not a panic"
        );
    }

    /// `render_erased_type_mismatch_produces_data_unavailable` asserts that
    /// `render_erased` returns `DataUnavailable` when passed the wrong type
    ///
    #[test]
    fn render_erased_type_mismatch_produces_data_unavailable() {
        let wrong: Box<dyn Any + Send> = Box::new(99_u8);
        let err = DoubleService
            .render_erased(wrong.as_ref(), &Colors::new(false))
            .unwrap_err();
        assert!(
            matches!(err, AppError::DataUnavailable(_)),
            "type mismatch must produce DataUnavailable, got: {err}"
        );
    }

    /// `render_erased_error_message_names_expected_type` asserts that the error
    /// message names the expected type
    ///
    #[test]
    fn render_erased_error_message_names_expected_type() {
        let wrong: Box<dyn Any + Send> = Box::new(99_u8);
        let err = DoubleService
            .render_erased(wrong.as_ref(), &Colors::new(false))
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("DoubleData"),
            "error message must name the expected type; got: {msg}"
        );
    }
}
