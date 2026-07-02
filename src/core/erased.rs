use std::any::Any;

use crate::core::error::AppError;
use crate::presentation::colors::Colors;
use crate::services::Service;

/// Result of an [`ErasedService::collect`] call: the concrete `Service::Data`, type-erased
/// behind `Any` so services with different `Data` types can share one collection
pub type CollectResult = Result<Box<dyn Any + Send + Sync>, AppError>;

/// `ErasedService` is an object-safe wrapper around [`Service`], letting services with different
/// associated `Data` types live together in a single `Vec<Box<dyn ErasedService>>`.
pub trait ErasedService: Send + Sync {
    /// `collect_erased()` delegates to the wrapped service's [`Service::collect`], boxing the
    /// result
    ///
    fn collect_erased(&self) -> CollectResult;

    /// `render_erased()` downcasts `data` back to the wrapped service's concrete `Data` type and
    /// delegates to [`Service::render`]
    ///
    fn render_erased(
        &self,
        label: &str,
        data: &(dyn Any + Send + Sync),
        colors: &Colors,
    ) -> Result<(), AppError>;
}

impl<T> ErasedService for T
where
    T: Service + Send + Sync,
    T::Data: Send + Sync + 'static,
{
    fn collect_erased(&self) -> CollectResult {
        Service::collect(self).map(|data| Box::new(data) as Box<dyn Any + Send + Sync>)
    }

    fn render_erased(
        &self,
        label: &str,
        data: &(dyn Any + Send + Sync),
        colors: &Colors,
    ) -> Result<(), AppError> {
        let data = data.downcast_ref::<T::Data>().expect(
            "ErasedService: collect_erased()/render_erased() type mismatch for the same \
             concrete service — this would indicate a bug in the blanket impl itself, not in \
             any service file",
        );
        Service::render(self, label, data, colors)
    }
}
