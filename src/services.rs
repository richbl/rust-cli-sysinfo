pub mod cpu;
pub mod disk;
pub mod memory;
pub mod system;
pub mod users;

use crate::core::error::AppError;
use crate::presentation::colors::Colors;

/// ` Service` is the common interface implemented by every system information service
pub trait Service {
    type Data;

    /// `collect()` reads raw system data and returns it
    ///
    fn collect(&self) -> Result<Self::Data, AppError>;

    /// `render()` formats and prints `data` to stdout using the provided colors
    ///
    fn render(&self, data: &Self::Data, colors: &Colors);
}
