//! Rust CLI System Information Utility (RCS)
//! Displays configurable system metric services natively on the command line

mod cli;
mod constants;
mod core;
mod presentation;
mod services;
mod slot;

use std::io::{self, Write};

use crate::cli::Opts;
use crate::constants::{APP_NAME, CLEAR_LINE, CLEAR_SCREEN, INDENT, SEP};
use crate::core::context::ServiceContext;
use crate::core::erased::CollectResult;
use crate::core::error::AppError;
use crate::core::registry::ServiceRegistry;
use crate::presentation::colors::Colors;
use crate::presentation::format::print_row_error;

/// `render_service_error()` renders an error message for a single service
///
fn render_service_error(idx: usize, registry: &ServiceRegistry, error: &AppError, colors: &Colors) {
    let label = registry.meta(idx).label;
    let value = format!("n/a - {error}");
    print_row_error(label, &value, colors);
}

/// `render_labeled()` displays a list of available service tokens
///
fn render_labeled(registry: &ServiceRegistry, c: &Colors) {
    let max_token_len = registry
        .all_meta()
        .map(|m| m.token.len())
        .max()
        .unwrap_or(4);

    println!("\n{INDENT}{}{}{}\n{INDENT}{}{}", c.bold, c.cyan, APP_NAME, SEP, c.reset);
    println!(
        "{INDENT}To configure the services displayed, separate each service token with a\n{INDENT}hyphen (-) in the desired order.\n"
    );
    println!("{INDENT}Available service tokens:\n");

    for meta in registry.all_meta() {
        println!(
            "    {}{:<width$}{}  {}",
            c.cyan,
            meta.token,
            c.reset,
            meta.description,
            width = max_token_len,
        );
    }

    println!(
        "\n{INDENT}Example:\n    {} -s {}OS-CPU-GPU-HST-KNL-DSKU{} -d /boot/efi",
        env!("CARGO_PKG_NAME"),
        c.cyan,
        c.reset,
    );

    println!("{INDENT}{}{}{}{}", c.bold, c.cyan, SEP, c.reset);
}

/// `collect_services()` gathers data for each unique active service index
///
fn collect_services(active: &[usize], registry: &ServiceRegistry) -> Vec<(usize, CollectResult)> {
    let mut collected: Vec<(usize, CollectResult)> = Vec::with_capacity(active.len());

    for &idx in active {
        // Avoid duplicate collections (e.g. CPU sampling delays) if the same slot is listed twice
        if !collected.iter().any(|(i, _)| *i == idx) {
            let service = registry.service(idx);
            collected.push((idx, service.collect_erased()));
        }
    }

    collected
}

/// `render_services()` iterates through the active indices and displays their collected data
///
fn render_services(
    active: &[usize],
    registry: &ServiceRegistry,
    collected: &[(usize, CollectResult)],
    colors: &Colors,
) {
    println!(
        "{INDENT}{}{}{}\n{INDENT}{}{}",
        colors.bold, colors.cyan, APP_NAME, SEP, colors.reset
    );

    // Render each unique active index
    for &idx in active {
        let Some((_, result)) = collected.iter().find(|(i, _)| *i == idx) else {
            render_service_error(
                idx,
                registry,
                &AppError::DataUnavailable("result not collected".into()),
                colors,
            );
            continue;
        };

        match result {
            Err(e) => render_service_error(idx, registry, e, colors),
            Ok(data) => {
                let service = registry.service(idx);
                let label = registry.meta(idx).label;

                // This only occurs if a real-time rendering failure occurs
                if let Err(e) = service.render_erased(label, &**data, colors) {
                    render_service_error(idx, registry, &e, colors);
                }
            }
        }
    }

    println!("{INDENT}{}{}{}", colors.cyan, SEP, colors.reset);
}

/// `main()` is the entry point for the utility (like... duh!)
///
fn main() {
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color);

    let ctx = ServiceContext::from(&opts);
    let registry = ServiceRegistry::new(&ctx);

    let Some(active) = opts.slot_filter.resolve(&registry, cli::fail_unknown_token) else {
        render_labeled(&registry, &colors);
        return;
    };

    if opts.clear {
        print!("{CLEAR_SCREEN}");
    }

    print!(
        "\n{INDENT}{}{}Just a moment...{}",
        colors.bold, colors.cyan, colors.reset
    );
    let _ = io::stdout().flush();

    let collected = collect_services(&active, &registry);

    if opts.clear {
        println!("{CLEAR_SCREEN}");
    } else {
        print!("{CLEAR_LINE}");
    }

    render_services(&active, &registry, &collected, &colors);
}
