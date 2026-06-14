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
    pub use crate::presentation::colors::{Colors, Threshold};
    pub use crate::presentation::format::print_row;
}

/// `Service` is the common interface implemented by every system information service
pub trait Service {
    type Data;

    /// `collect()` reads raw system data and returns it
    ///
    fn collect(&self) -> Result<Self::Data, AppError>;

    /// `render()` formats and prints `data` to stdout using the provided colors
    ///
    fn render(&self, data: &Self::Data, colors: &Colors);
}

/// `AnyService` is an object-safe, type-erased counterpart to [`Service`]
///
pub trait AnyService {
    /// `collect_erased()` calls the underlying service's `collect()`, boxing the
    /// successful result as `Box<dyn Any>`
    ///
    fn collect_erased(&self) -> Result<Box<dyn Any>, AppError>;

    /// `render_erased()` downcasts `data` back to the underlying service's `Data` type
    /// and calls `render()`
    ///
    fn render_erased(&self, data: &dyn Any, colors: &Colors);
}

/// Blanket implementation: every [`Service`] is automatically an [`AnyService`]
impl<S> AnyService for S
where
    S: Service,
    S::Data: 'static,
{
    /// `collect_erased()` boxes the successful result of `collect()`
    ///
    fn collect_erased(&self) -> Result<Box<dyn Any>, AppError> {
        self.collect().map(|data| Box::new(data) as Box<dyn Any>)
    }

    /// `render_erased()` downcasts `data` back to the underlying service's `Data` type
    /// and calls `render()`
    ///
    fn render_erased(&self, data: &dyn Any, colors: &Colors) {
        let data = data
            .downcast_ref::<S::Data>()
            .expect("AnyService: collect/render data type mismatch");
        self.render(data, colors);
    }
}
