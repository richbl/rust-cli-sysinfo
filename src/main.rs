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
use crate::constants::{CLEAR_LINE, CLEAR_SCREEN, INDENT, LABEL_WIDTH, SEP_FALLBACK};
use crate::core::context::ServiceContext;
use crate::core::erased::CollectResult;
use crate::core::registry::ServiceRegistry;
use crate::core::utils::generate_title;

use crate::presentation::colors::Colors;
use crate::presentation::format::{RenderedRow, Threshold, print_row};

/// `render_labeled()` displays a list of available service tokens
///
fn render_labeled(registry: &ServiceRegistry, c: &Colors) {
    let max_token_len = registry
        .all_meta()
        .map(|m| m.token.len())
        .max()
        .unwrap_or(4);

    println!(
        "\n{INDENT}{}{}{}\n{INDENT}{}",
        c.bold,
        c.cyan,
        generate_title(SEP_FALLBACK.chars().count()),
        c.reset
    );
    println!(
        "{INDENT}To configure the services displayed, separate each service token with a\n{INDENT}hyphen (-) in the desired order.\n"
    );
    println!("{INDENT}Available service tokens (default order):\n");

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
        "\n{INDENT}Example:\n    {} -s {}OS-CPU-GPU-HST-KNL-DSKU{} -d /boot/efi\n",
        env!("CARGO_PKG_NAME"),
        c.cyan,
        c.reset,
    );

    println!("{INDENT}{}{}{}{}", c.bold, c.cyan, SEP_FALLBACK, c.reset);
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
    let active_rows = build_active_rows(active, registry, collected);
    let sep_len = calculate_separator_length(&active_rows);

    print_service_table_header(sep_len, colors);
    for (label, row) in active_rows {
        print_row(label, &row.value, &row.threshold, colors);
    }
    print_table_footer(sep_len, colors);
}
/// `build_active_rows()` builds a list of active services and their rendered rows
///
fn build_active_rows(
    active: &[usize],
    registry: &ServiceRegistry,
    collected: &[(usize, CollectResult)],
) -> Vec<(&'static str, RenderedRow)> {
    active
        .iter()
        .map(|&idx| {
            let meta = registry.meta(idx);
            let service = registry.service(idx);
            let row = match collected.iter().find(|(i, _)| *i == idx) {
                None => RenderedRow {
                    value: "n/a - result not collected".to_string(),
                    threshold: Threshold::Error,
                },
                Some((_, Err(e))) => RenderedRow {
                    value: format!("n/a - {e}"),
                    threshold: Threshold::Error,
                },
                Some((_, Ok(data))) => match service.render_erased(&**data) {
                    Err(e) => RenderedRow {
                        value: format!("n/a - {e}"),
                        threshold: Threshold::Error,
                    },
                    Ok(rendered_row) => rendered_row,
                },
            };

            (meta.label, row)
        })
        .collect()
}

/// `calculate_separator_length()` calculates the length of the separator line
/// based on the longest value string in `active_rows`
///
fn calculate_separator_length(active_rows: &[(&'static str, RenderedRow)]) -> usize {
    let max_value_len = active_rows
        .iter()
        .flat_map(|(_, row)| row.value.lines())
        .map(|line| line.trim().len())
        .max()
        .unwrap_or(0);

    LABEL_WIDTH + 3 + max_value_len
}

/// `print_service_table_header()` prints the table header
///
fn print_service_table_header(sep_len: usize, colors: &Colors) {
    println!(
        "{INDENT}{}{}{}\n",
        colors.bold,
        colors.cyan,
        generate_title(sep_len)
    );
    print!("{}", colors.reset);
}

/// `print_table_footer()` prints the table footer
///
fn print_table_footer(sep_len: usize, colors: &Colors) {
    println!(
        "\n{INDENT}{}{}{}",
        colors.cyan,
        "─".repeat(sep_len),
        colors.reset
    );
}

/// `main()` is the entry point for the utility
///
fn main() {
    let opts = Opts::from_args();
    let colors = Colors::new(opts.color, opts.service_key_color.as_deref());

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
