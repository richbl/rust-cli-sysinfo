//! Linux CLI System Information Utility
//! Displays metrics natively from Linux-based system calls
//!

mod cli;
mod core;
mod presentation;
mod services;
mod slot;

use std::io::{self, Write};

use crate::cli::Opts;
use crate::core::error::AppError;
use crate::presentation::colors::Colors;
use crate::services::{
    Service,
    cpu_model::{CpuModelInfo, CpuModelService},
    cpu_usage::{CpuUsageInfo, CpuUsageService},
    disk::{DiskInfo, DiskService},
    gpu::{GpuInfo, GpuService},
    hostname::{HostnameInfo, HostnameService},
    kernel::{KernelInfo, KernelService},
    load_avg::{LoadAvgInfo, LoadAvgService},
    memory::{MemInfo, MemoryService},
    os_name::{OsInfo, OsService},
    uptime::{UptimeInfo, UptimeService},
    users::{UsersInfo, UsersService},
};
use crate::slot::{ServiceSlot, SlotFilter};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Rust-CLI-SysInfo";
pub const SEP: &str =
    "────────────────────────────────────────────────────────────────────────────────";

/// `Services` struct holds one instance of every service
struct Services {
    os: OsService,
    hostname: HostnameService,
    cpu_model: CpuModelService,
    gpu: GpuService,
    kernel: KernelService,
    uptime: UptimeService,
    load_avg: LoadAvgService,
    cpu_usage: CpuUsageService,
    memory: MemoryService,
    disk: DiskService,
    users: UsersService,
}

/// `CollectedData` holds the result of every service `collect()` call
///
struct CollectedData {
    os: Option<Result<OsInfo, AppError>>,
    hostname: Option<Result<HostnameInfo, AppError>>,
    cpu_model: Option<Result<CpuModelInfo, AppError>>,
    gpu: Option<Result<GpuInfo, AppError>>,
    kernel: Option<Result<KernelInfo, AppError>>,
    uptime: Option<Result<UptimeInfo, AppError>>,
    load_avg: Option<Result<LoadAvgInfo, AppError>>,
    cpu_usage: Option<Result<CpuUsageInfo, AppError>>,
    memory: Option<Result<MemInfo, AppError>>,
    disk: Option<Result<DiskInfo, AppError>>,
    users: Option<Result<UsersInfo, AppError>>,
}

/// `render_if_ok()` renders collected service data, or prints a warning on failure/skip
///
fn render_if_ok<T>(slot_name: &str, data: Option<&Result<T, AppError>>, render: impl FnOnce(&T)) {
    match data.and_then(|r| r.as_ref().ok()) {
        Some(d) => render(d),
        None => eprintln!("Warning: Failed to collect {slot_name} metrics"),
    }
}

impl Services {
    /// `new()` constructs all services, wiring in user-supplied options
    ///
    fn new(opts: &Opts) -> Self {
        Self {
            os: OsService,
            hostname: HostnameService,
            cpu_model: CpuModelService,
            gpu: GpuService,
            kernel: KernelService,
            uptime: UptimeService,
            load_avg: LoadAvgService,
            cpu_usage: CpuUsageService {
                sample_ms: opts.cpu_sample_ms,
            },
            memory: MemoryService,
            disk: DiskService {
                mount: opts.disk_mount.clone(),
            },
            users: UsersService,
        }
    }

    /// `collect()` runs only the services whose slots appear in `active_slots`
    ///
    fn collect(&self, active_slots: &[ServiceSlot]) -> CollectedData {
        let active = |slot: ServiceSlot| active_slots.contains(&slot);

        CollectedData {
            os: active(ServiceSlot::Os).then(|| self.os.collect()),
            hostname: active(ServiceSlot::Hst).then(|| self.hostname.collect()),
            cpu_model: active(ServiceSlot::CpuM).then(|| self.cpu_model.collect()),
            gpu: active(ServiceSlot::Gpu).then(|| self.gpu.collect()),
            kernel: active(ServiceSlot::Knl).then(|| self.kernel.collect()),
            uptime: active(ServiceSlot::Upt).then(|| self.uptime.collect()),
            load_avg: active(ServiceSlot::Load).then(|| self.load_avg.collect()),
            cpu_usage: active(ServiceSlot::Cpu).then(|| self.cpu_usage.collect()),
            memory: active(ServiceSlot::Ram).then(|| self.memory.collect()),
            disk: active(ServiceSlot::Dsk).then(|| self.disk.collect()),
            users: active(ServiceSlot::Usr).then(|| self.users.collect()),
        }
    }

    /// `render_slot()` renders a single slot from its collected data
    ///
    fn render_slot(&self, slot: ServiceSlot, data: &CollectedData, c: &Colors) {
        match slot {
            ServiceSlot::Os => render_if_ok("OS", data.os.as_ref(), |d| self.os.render(d, c)),
            ServiceSlot::Hst => {
                render_if_ok("Hostname", data.hostname.as_ref(), |d| {
                    self.hostname.render(d, c);
                });
            }
            ServiceSlot::CpuM => render_if_ok("CPU model", data.cpu_model.as_ref(), |d| {
                self.cpu_model.render(d, c);
            }),
            ServiceSlot::Gpu => render_if_ok("GPU", data.gpu.as_ref(), |d| self.gpu.render(d, c)),
            ServiceSlot::Knl => {
                render_if_ok("Kernel", data.kernel.as_ref(), |d| self.kernel.render(d, c));
            }
            ServiceSlot::Upt => {
                render_if_ok("Uptime", data.uptime.as_ref(), |d| self.uptime.render(d, c));
            }
            ServiceSlot::Load => render_if_ok("Load Average", data.load_avg.as_ref(), |d| {
                self.load_avg.render(d, c);
            }),
            ServiceSlot::Cpu => render_if_ok("CPU usage", data.cpu_usage.as_ref(), |d| {
                self.cpu_usage.render(d, c);
            }),
            ServiceSlot::Ram => {
                render_if_ok("Memory", data.memory.as_ref(), |d| self.memory.render(d, c));
            }
            ServiceSlot::Dsk => {
                render_if_ok("Disk", data.disk.as_ref(), |d| self.disk.render(d, c));
            }
            ServiceSlot::Usr => {
                render_if_ok("Users", data.users.as_ref(), |d| self.users.render(d, c));
            }
        }
    }

    /// `render()` renders all active slots in order
    ///
    fn render(&self, slots: &[ServiceSlot], data: &CollectedData, c: &Colors) {
        println!("  {}{}{}\n  {}{}", c.bold, c.cyan, APP_NAME, SEP, c.reset);

        for &slot in slots {
            self.render_slot(slot, data, c);
        }

        println!("  {}{}{}", c.cyan, SEP, c.reset);
    }

    /// `render_labeled()` prints the token reference table (output of `-s` with no argument)
    ///
    fn render_labeled(c: &Colors) {
        let max_token_len = ServiceSlot::ALL
            .iter()
            .map(|s| s.token().len())
            .max()
            .unwrap_or(4);

        println!("\n  {}{}{}\n  {}{}", c.bold, c.cyan, APP_NAME, SEP, c.reset);

        println!(
            "  To configure the services displayed, separate each services token with a\n  hyphen (-) in the desired order.\n"
        );

        println!("  Available services tokens:\n");

        // Loop over all services, printing their tokens and descriptions
        for slot in ServiceSlot::ALL {
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
            "\n  Example:\n    {} -s {}OS-CPUM-GPU-HST-KNL-DSK{} -d /boot/efi\n",
            env!("CARGO_PKG_NAME"),
            c.cyan,
            c.reset,
        );

        println!("  {}{}{}{}", c.bold, c.cyan, SEP, c.reset);
    }
}

/// `main()` parses CLI options, collects system data, and renders to stdout
///
fn main() {
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    // -s with no argument: print the token reference table and exit (no collection needed)
    if matches!(opts.slot_filter, SlotFilter::ShowLabeled) {
        Services::render_labeled(&colors);
        return;
    }

    let active_slots: Vec<ServiceSlot> = match &opts.slot_filter {
        SlotFilter::Default => ServiceSlot::ALL.to_vec(),
        SlotFilter::Custom(slots) => slots.clone(),
        SlotFilter::ShowLabeled => unreachable!(),
    };

    if opts.clear {
        print!("\x1bc");
    }

    print!(
        "\n  {}{}Just a moment...{}",
        colors.bold, colors.cyan, colors.reset
    );
    let _ = io::stdout().flush();

    let services = Services::new(&opts);
    let data = services.collect(&active_slots);

    if opts.clear {
        println!("\x1bc");
    } else {
        print!("\r\x1b[2K");
    }

    services.render(&active_slots, &data, &colors);
}
