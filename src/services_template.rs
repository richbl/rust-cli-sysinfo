use super::prelude::*;

/// This `services_template.rs` file serves as a very simple minimalist example of the pattern
/// used to create a new RCS service
///
/// 1. Make a copy of this file, and place it in the `/services` folder
/// 2. Edit this new service file, and save it
/// 3. Build RCS: the build process will look in the `/services` folder and include all service
///    files found there
///

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

    /// `render()` formats and returns `data` via a `RenderedRow`
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        Ok(RenderedRow {
            value: data.message.clone(),
            threshold: Threshold::None,
        })
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
        TemplateService.render(&data).unwrap();
    }
}
