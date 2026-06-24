//! Rust CLI System Information Utility (RCS)
//! Displays metrics natively from Linux-based system calls

mod cli;
mod constants;
mod core;
mod presentation;
mod services;
mod slot;

use std::any::Any;
use std::collections::HashMap;
use std::io::{self, Write};

use crate::cli::Opts;
use crate::constants::{APP_NAME, CLEAR_LINE, CLEAR_SCREEN, SEP};
use crate::core::error::AppError;
use crate::presentation::colors::Colors;
use crate::presentation::format::{Threshold, print_row};
use crate::services::AnyService;
use crate::services::cpu_model::CpuModelService;
use crate::services::cpu_usage::CpuUsageService;
use crate::services::disk::DiskService;
use crate::services::gpu::GpuService;
use crate::services::hostname::HostnameService;
use crate::services::kernel::KernelService;
use crate::services::load_avg::LoadAvgService;
use crate::services::memory::MemoryService;
use crate::services::os_name::OsService;
use crate::services::uptime::UptimeService;
use crate::services::users::UsersService;
use crate::slot::{ServiceSlot, SlotFilter};

/// `CollectResult` is the type-erased result of a single service's `collect()` call
type CollectResult = Result<Box<dyn Any + Send>, AppError>;

/// `ServiceEntry` pairs a [`ServiceSlot`] identifier with its type-erased service
/// implementation
struct ServiceEntry {
    service: Box<dyn AnyService>,
}

/// `ServiceEntry` implements `AnyService` via `Box<AnyService>`
impl ServiceEntry {
    /// `new()` boxes `service` behind the [`AnyService`] trait
    ///
    fn new(service: impl AnyService + 'static) -> Self {
        Self {
            service: Box::new(service),
        }
    }
}

/// `build_registry()` constructs one [`ServiceEntry`] for every known service, wiring
/// in user-supplied options
///
/// When adding/removing services, this is the one location in this file that needs to change...
/// Also, one entry in `slot::SLOT_TABLE` needs to be added and a new module created
/// under `services/`
///
fn build_registry(opts: &Opts) -> HashMap<ServiceSlot, ServiceEntry> {
    let mut map = HashMap::new();
    map.insert(ServiceSlot::Os, ServiceEntry::new(OsService));
    map.insert(ServiceSlot::Hst, ServiceEntry::new(HostnameService));
    map.insert(ServiceSlot::Cpu, ServiceEntry::new(CpuModelService));
    map.insert(ServiceSlot::Gpu, ServiceEntry::new(GpuService));
    map.insert(ServiceSlot::Knl, ServiceEntry::new(KernelService));
    map.insert(ServiceSlot::Upt, ServiceEntry::new(UptimeService));
    map.insert(ServiceSlot::Load, ServiceEntry::new(LoadAvgService));
    map.insert(
        ServiceSlot::CpuU,
        ServiceEntry::new(CpuUsageService {
            sample_ms: opts.cpu_sample_ms,
        }),
    );
    map.insert(ServiceSlot::RamU, ServiceEntry::new(MemoryService));
    map.insert(
        ServiceSlot::DskU,
        ServiceEntry::new(DiskService {
            mount: opts.disk_mount.clone(),
        }),
    );
    map.insert(ServiceSlot::Usr, ServiceEntry::new(UsersService));
    map
}

/// `render_service_error()` prints a standard error row for a slot whose data could
/// not be collected or rendered
///
fn render_service_error(id: ServiceSlot, error: &AppError, colors: &Colors) {
    let label = format!("  {}:", id.token());
    let value = format!("unavailable ({error})");
    print_row(&label, &value, &Threshold::None, colors);
}

/// `render_labeled()` prints the token reference table (output of `-s` with no argument)
///
fn render_labeled(c: &Colors) {
    let slots = ServiceSlot::all();
    let max_token_len = slots.iter().map(|s| s.token().len()).max().unwrap_or(4);

    println!("\n  {}{}{}\n  {}{}", c.bold, c.cyan, APP_NAME, SEP, c.reset);
    println!(
        "  To configure the services displayed, separate each service token with a\n  hyphen (-) in the desired order.\n"
    );
    println!("  Available service tokens:\n");

    // Loop over all services, printing their tokens and descriptions
    for slot in &slots {
        println!(
            "    {}{:<width$}{}  {}",
            c.cyan,
            slot.token(),
            c.reset,
            slot.description(),
            width = max_token_len,
        );
    }

    println!(
        "\n  Example:\n    {} -s {}OS-CPU-GPU-HST-KNL-DSKU{} -d /boot/efi",
        env!("CARGO_PKG_NAME"),
        c.cyan,
        c.reset,
    );

    println!("  {}{}{}{}", c.bold, c.cyan, SEP, c.reset);
}

/// `collect_services()` gathers data for each unique active slot
///
fn collect_services(
    active_slots: &[ServiceSlot],
    registry: &HashMap<ServiceSlot, ServiceEntry>,
) -> HashMap<ServiceSlot, CollectResult> {
    let mut collected: HashMap<ServiceSlot, CollectResult> = HashMap::new();

    for &id in active_slots {
        collected.entry(id).or_insert_with(|| {
            registry.get(&id).map_or_else(
                || {
                    Err(AppError::DataUnavailable(format!(
                        "no registry entry for slot '{}'",
                        id.token()
                    )))
                },
                |entry| entry.service.collect_erased(),
            )
        });
    }

    collected
}

/// `render_services()` iterates through the active slots and displays their collected
/// data using the provided colors
///
fn render_services(
    active_slots: &[ServiceSlot],
    registry: &HashMap<ServiceSlot, ServiceEntry>,
    collected: &HashMap<ServiceSlot, CollectResult>,
    colors: &Colors,
) {
    println!(
        "  {}{}{}\n  {}{}",
        colors.bold, colors.cyan, APP_NAME, SEP, colors.reset
    );

    for &id in active_slots {
        let Some(result) = collected.get(&id) else {
            render_service_error(
                id,
                &AppError::DataUnavailable("result not collected".into()),
                colors,
            );
            continue;
        };

        match result {
            Err(e) => render_service_error(id, e, colors),
            Ok(data) => {
                let Some(entry) = registry.get(&id) else {
                    render_service_error(
                        id,
                        &AppError::DataUnavailable("no registry entry".into()),
                        colors,
                    );
                    continue;
                };

                if let Err(e) = entry.service.render_erased(data.as_ref(), colors) {
                    render_service_error(id, &e, colors);
                }
            }
        }
    }

    println!("  {}{}{}", colors.cyan, SEP, colors.reset);
}

/// `main()` parses CLI options, collects system data, and renders to stdout
///
fn main() {
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    // -s with no argument: print the token reference table and exit
    if matches!(opts.slot_filter, SlotFilter::ShowLabeled) {
        render_labeled(&colors);
        return;
    }

    let active_slots: Vec<ServiceSlot> = match &opts.slot_filter {
        SlotFilter::Default => ServiceSlot::all(),
        SlotFilter::Custom(slots) => slots.clone(),
        SlotFilter::ShowLabeled => unreachable!(),
    };

    if opts.clear {
        print!("{CLEAR_SCREEN}");
    }

    print!(
        "\n  {}{}Just a moment...{}",
        colors.bold, colors.cyan, colors.reset
    );
    let _ = io::stdout().flush();

    let registry = build_registry(&opts);
    let collected = collect_services(&active_slots, &registry);

    if opts.clear {
        println!("{CLEAR_SCREEN}");
    } else {
        print!("{CLEAR_LINE}");
    }

    render_services(&active_slots, &registry, &collected, &colors);
}
