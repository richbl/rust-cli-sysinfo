/// `ServiceMeta` is the small, `static` piece of display metadata every service supplies from
/// its own `descriptor()` function
#[derive(Clone, Copy, Debug)]
pub struct ServiceMeta {
    /// Short uppercase token used with `-s`/`--services` (e.g. `"CPU"`)
    pub token: &'static str,

    /// Left-column label printed before the service's rendered value (e.g. `indented_label!("CPU:")`).
    /// Use the `indented_label!` macro so the indent is driven by `constants::INDENT`.
    pub label: &'static str,

    /// One-line description shown in the `-s` (no argument) reference table.
    pub description: &'static str,

    /// Sort key controlling default display order (ascending, ties broken by discovery order)
    /// which exists solely so that renaming or adding service files never reshuffles the
    /// CLI output order
    pub sort_order: u16,
}
