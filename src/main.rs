//! Rust CLI System Information Utility (RCS)
//! Displays metrics natively from Linux-based system calls

mod cli;
mod constants;
mod core;
mod presentation;
mod services;
mod slot;

// Replace occurrences of std::collections::HashMap with Vec
use std::io::{self, Write};

use crate::cli::Opts;
use crate::constants::{APP_NAME, CLEAR_LINE, CLEAR_SCREEN, SEP};
use crate::core::context::ServiceContext;
use crate::core::error::AppError;
use crate::presentation::colors::Colors;
use crate::presentation::format::print_row_error;
use crate::services::registry::{CollectResult, ServiceRegistry};
use crate::slot::ServiceSlot;

/// `render_service_error` renders an error message for a single service
///
fn render_service_error(id: ServiceSlot, error: &AppError, colors: &Colors) {
    let label = id.label();
    let value = format!("n/a - {error}");
    print_row_error(label, &value, colors);
}

/// `render_labeled()` displays a list of available service tokens
///
fn render_labeled(c: &Colors) {
    let slots = ServiceSlot::all();
    let max_token_len = slots.iter().map(|s| s.token().len()).max().unwrap_or(4);

    println!("\n  {}{}{}\n  {}{}", c.bold, c.cyan, APP_NAME, SEP, c.reset);
    println!(
        "  To configure the services displayed, separate each service token with a\n  hyphen (-) in the desired order.\n"
    );
    println!("  Available service tokens:\n");

    for slot in slots {
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
    registry: &ServiceRegistry,
) -> Vec<(ServiceSlot, CollectResult)> {
    let mut collected: Vec<(ServiceSlot, CollectResult)> = Vec::with_capacity(active_slots.len());

    for &id in active_slots {
        // Avoid duplicate collections (e.g. CPU sampling delays) if the same slot is listed twice
        if !collected.iter().any(|(slot, _)| *slot == id) {
            let service = registry.get(id);
            collected.push((id, service.collect()));
        }
    }

    collected
}

/// `render_services()` iterates through the active slots and displays their collected data
///
fn render_services(
    active_slots: &[ServiceSlot],
    registry: &ServiceRegistry,
    collected: &[(ServiceSlot, CollectResult)],
    colors: &Colors,
) {
    println!(
        "  {}{}{}\n  {}{}",
        colors.bold, colors.cyan, APP_NAME, SEP, colors.reset
    );

    // Render each unique active slot
    for &id in active_slots {
        let Some((_, result)) = collected.iter().find(|(slot, _)| *slot == id) else {
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
                let service = registry.get(id);

                // This only occurs if a real-time rendering failure occurs
                if let Err(e) = service.render(id.label(), data, colors) {
                    render_service_error(id, &e, colors);
                }
            }
        }
    }

    println!("  {}{}{}", colors.cyan, SEP, colors.reset);
}

/// `main()` is the entry point for the utility (like... duh!)
///
fn main() {
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    let Some(active_slots) = opts.slot_filter.to_active_slots() else {
        render_labeled(&colors);
        return;
    };

    if opts.clear {
        print!("{CLEAR_SCREEN}");
    }

    print!(
        "\n  {}{}Just a moment...{}",
        colors.bold, colors.cyan, colors.reset
    );
    let _ = io::stdout().flush();

    let ctx = ServiceContext::from(&opts);
    let registry = ServiceRegistry::new(&ctx);
    let collected = collect_services(&active_slots, &registry);

    if opts.clear {
        println!("{CLEAR_SCREEN}");
    } else {
        print!("{CLEAR_LINE}");
    }

    render_services(&active_slots, &registry, &collected, &colors);
}
