//! Linux CLI System Information Utility
//! Displays metrics natively from Linux-based system calls
//!
//! Flags: [-d|--disk <path>] [-c|--cpu-sample-rate <ms>] [-n, --no-clear] [-o,--no-color] [-h|--help]

mod cli;
mod core;
mod presentation;
mod services;

use std::io::{self, Write};

use crate::cli::Opts;
use crate::core::error::AppError;
use crate::presentation::colors::Colors;
use crate::services::{
    Service,
    cpu::{CpuInfo, CpuService},
    disk::{DiskInfo, DiskService},
    memory::{MemInfo, MemoryService},
    system::{SystemInfo, SystemService},
    users::{UsersInfo, UsersService},
};

/// Application version sourced directly from `Cargo.toml`
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Rust-CLI-SysInfo";

const SEP: &str = "────────────────────────────────────────────────────────────";

/// `Services` aggregates all data-collection services used by the utility
struct Services {
    sys: SystemService,
    cpu: CpuService,
    mem: MemoryService,
    disk: DiskService,
    users: UsersService,
}

/// `CollectedData` manages the utility services results
struct CollectedData {
    sys: SystemInfo,
    cpu: Result<CpuInfo, AppError>,
    mem: Result<MemInfo, AppError>,
    disk: Result<DiskInfo, AppError>,
    users: Result<UsersInfo, AppError>,
}

/// `Services` aggregates all data-collection services used by the utility
impl Services {
    /// `new()` constructs utility services, wiring in user-supplied options
    ///
    fn new(opts: &Opts) -> Self {
        Self {
            sys: SystemService,
            cpu: CpuService {
                sample_ms: opts.cpu_sample_ms,
            },
            mem: MemoryService,
            disk: DiskService {
                mount: opts.disk_mount.clone(),
            },
            users: UsersService,
        }
    }

    /// `collect()` runs all utility services and returns the collected data
    ///
    fn collect(&self) -> CollectedData {
        CollectedData {
            sys: self
                .sys
                .collect()
                .expect("Critical failure collecting system data"),
            cpu: self.cpu.collect(),
            mem: self.mem.collect(),
            disk: self.disk.collect(),
            users: self.users.collect(),
        }
    }

    /// `render()` renders the full utility, printing each section with optional color
    ///
    fn render(&self, data: &CollectedData, colors: &Colors) {
        println!(
            "  {}{}{}{}",
            colors.bold, colors.cyan, APP_NAME, colors.reset
        );

        println!("  {}{}{}", colors.cyan, SEP, colors.reset);

        self.sys.render(&data.sys, colors);

        match &data.cpu {
            Ok(d) => self.cpu.render(d, colors),
            Err(_) => eprintln!("Warning: Failed to collect CPU metrics"),
        }

        match &data.mem {
            Ok(d) => self.mem.render(d, colors),
            Err(_) => eprintln!("Warning: Failed to collect Memory metrics"),
        }

        match &data.disk {
            Ok(d) => self.disk.render(d, colors),
            Err(_) => eprintln!("Warning: Failed to collect Disk metrics"),
        }

        match &data.users {
            Ok(d) => self.users.render(d, colors),
            Err(_) => eprintln!("Warning: Failed to collect Users metrics"),
        }

        println!("  {}{}{}", colors.cyan, SEP, colors.reset);
    }
}

/// `main()` parses CLI options, collects system data, and renders the utility
///
fn main() {
    // Parse command-line options and initialize colors
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    // Clear the screen if requested
    if opts.clear {
        print!("\x1bc");
    }

    // Display a pending message (is the AE-35 unit is failing?)
    print!(
        "\n  {}{}Just a moment...{}",
        colors.bold, colors.cyan, colors.reset
    );
    let _ = io::stdout().flush();

    // Initialize services
    let services = Services::new(&opts);

    // Collect data (this will block during CPU sampling)
    let data = services.collect();

    // Clear the loading message (or the whole screen if requested)
    if opts.clear {
        println!("\x1bc");
    } else {
        print!("\r\x1b[2K");
    }

    // Render the thing already!
    services.render(&data, &colors);
}
