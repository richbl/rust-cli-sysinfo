//! TEMPLATE SERVICE — copy this file to add a new service to the utility
//!
//! Every `.rs` file dropped into `src/services` is discovered automatically by `build.rs` and
//! wired into the binary at compile time. Nothing outside this file needs to change
//!
//!   1. Copy this file to `src/services/<your_service_name>.rs`
//!   2. Rename `TemplateInfo` / `TemplateService` to match your service
//!   3. Implement `collect()` to gather your data and `render()` to print it
//!   4. Update the `ServiceMeta` fields in `descriptor()` below (token, label via
//!      `indented_label!("Your label:")`, description, display order)
//!   5. `cargo build` and that's it. Your service now appears in the default output, in
//!      `-s <TOKEN>` selections, and in the `-s` reference table
//!   6. Delete this comment block
//!

use super::prelude::*;

/// `TemplateInfo` is the data your service collects. It can be anything — a `String`, numeric
/// metrics, a `Vec` of results, etc. It doesn't need to implement any trait; `Service::Data`
/// only requires it to be a plain type. Note that it does need to be `Send + Sync + 'static` for
/// the registry's type-erasure to work
pub struct TemplateInfo {
    pub message: String,
}

/// `TemplateService` is the unit type (or struct, if it needs fields — see `DiskService` or
/// `CpuUsageService` for examples that hold configuration read from `ServiceContext`) that
/// implements the collection/rendering logic
pub struct TemplateService;

/// `TemplateService` implements the `Service` trait — the only interface every service must
/// satisfy. `collect()` and `render()` are deliberately separate:
///   - `collect()` should do the (possibly fallible) work of reading real system state
///   - `render()` should be a simple, non-fallible formatting step. Keeping them separate is what
///     lets `main.rs` show a "Just a moment..." spinner while every service's `collect()` runs,
///     then renders everything all at once
impl Service for TemplateService {
    type Data = TemplateInfo;

    /// `collect()` reads raw data and returns it, or an `AppError` if the data could not be
    /// obtained
    ///
    ///  Services typically read from `/proc`, `/sys`, or a system call — see
    /// `hostname.rs` or `kernel.rs` for some examples
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(TemplateInfo {
            message: "This is a service template (example)".to_string(),
        })
    }

    /// `render()` formats and prints `data` to stdout via `print_row()`, which handles the
    /// left-aligned label column
    ///
    /// Use `Threshold::Check { value, warn, crit }` instead of `Threshold::None` if your value
    /// should be color-coded (see `memory.rs` or `disk.rs`)
    ///
    fn render(&self, label: &str, data: &Self::Data, c: &Colors) -> Result<(), AppError> {
        print_row(label, &data.message, &Threshold::None, c);
        Ok(())
    }
}

/// `descriptor()` is the one required entry point `build.rs` looks for in every file under
/// `src/services`
///
/// `ctx` carries configuration parsed from CLI flags (e.g. disk mount, CPU sample rate) —
/// ignore it (as in this example) if your service doesn't need any, or read from it like
/// `DiskService::new` or `CpuUsageService::new`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            // Short, uppercase, unique across all services
            token: "TPL",
            // Left-column label, printed before the rendered value
            label: "Template",
            // One-line description shown in the `-s` (no argument) reference table
            description: "Service template (example)",
            // Sort key controlling default display order (ascending). The default core services
            // use 0-10
            sort_order: 999,
        },
        Box::new(TemplateService),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::colors::Colors;

    /// `collect_returns_ok()` asserts that the template service's `collect()` always succeeds —
    /// a real service's equivalent test would assert something meaningful about the data
    /// collected from the system instead
    ///
    #[test]
    fn collect_returns_ok() {
        assert!(TemplateService.collect().is_ok());
    }

    /// `render_does_not_panic()` asserts that rendering the collected data will not panic
    ///
    #[test]
    fn render_does_not_panic() {
        let data = TemplateService.collect().unwrap();
        TemplateService
            .render("Template", &data, &Colors::new(false))
            .unwrap();
    }
}
