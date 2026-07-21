use crate::constants::{BOLD, CYAN, GREEN, RED, RESET, YELLOW};

/// ANSI escape sequences for terminal color output
///
pub struct Colors {
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub cyan: &'static str,
    pub bold: &'static str,

    /// Resets all attributes back to the terminal default
    pub reset: &'static str,

    /// ANSI foreground color for service label keys; either a 24-bit true-color escape built
    /// from the user's `-k/--service-key-color` hex value (or `cyan` by default)
    pub service_key: String,
}

/// `Colors` constructor
impl Colors {
    /// `new` creates a `Colors` instance with ANSI codes when `enabled` is `true`, or empty
    /// strings otherwise
    ///
    /// `key_color` is an optional pre-validated `#RRGGBB` hex string supplied by the
    /// `-k/--service-key-color` CLI flag
    ///
    #[must_use]
    pub fn new(enabled: bool, key_color: Option<&str>) -> Self {
        if enabled {
            let service_key = key_color
                .and_then(hex_to_ansi_fg)
                .unwrap_or_else(|| CYAN.to_string());

            Self {
                red: RED,
                green: GREEN,
                yellow: YELLOW,
                cyan: CYAN,
                bold: BOLD,
                reset: RESET,
                service_key,
            }
        } else {
            Self {
                red: "",
                green: "",
                yellow: "",
                cyan: "",
                bold: "",
                reset: "",
                service_key: String::new(),
            }
        }
    }
}

/// `hex_to_ansi_fg()` converts a validated `#RRGGBB` hex string to a 24-bit ANSI foreground
/// escape sequence (`\x1b[38;2;R;G;Bm`)
///
fn hex_to_ansi_fg(hex: &str) -> Option<String> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(format!("\x1b[38;2;{r};{g};{b}m"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Colors::new(true, None) tests

    /// `enabled_colors_are_non_empty()` asserts that ANSI color fields are non-empty when enabled
    ///
    #[test]
    fn enabled_colors_are_non_empty() {
        let c = Colors::new(true, None);
        assert!(!c.red.is_empty(), "red should be non-empty when enabled");
        assert!(
            !c.green.is_empty(),
            "green should be non-empty when enabled"
        );
        assert!(
            !c.yellow.is_empty(),
            "yellow should be non-empty when enabled"
        );
        assert!(!c.cyan.is_empty(), "cyan should be non-empty when enabled");
        assert!(!c.bold.is_empty(), "bold should be non-empty when enabled");
        assert!(
            !c.reset.is_empty(),
            "reset should be non-empty when enabled"
        );
        assert!(
            !c.service_key.is_empty(),
            "service_key should be non-empty when enabled"
        );
    }

    /// `enabled_colors_start_with_escape_sequence()` asserts that ANSI color fields start with the
    /// escape sequence when enabled
    ///
    #[test]
    fn enabled_colors_start_with_escape_sequence() {
        let c = Colors::new(true, None);
        // All ANSI codes must begin with ESC (0x1b)
        assert!(c.red.starts_with('\x1b'), "red must start with ESC");
        assert!(c.green.starts_with('\x1b'), "green must start with ESC");
        assert!(c.yellow.starts_with('\x1b'), "yellow must start with ESC");
        assert!(c.cyan.starts_with('\x1b'), "cyan must start with ESC");
        assert!(c.bold.starts_with('\x1b'), "bold must start with ESC");
        assert!(c.reset.starts_with('\x1b'), "reset must start with ESC");
        assert!(
            c.service_key.starts_with('\x1b'),
            "service_key must start with ESC when enabled"
        );
    }

    // Colors::new(false, None) test

    /// `disabled_colors_are_all_empty()` asserts that color fields are empty when disabled
    ///
    #[test]
    fn disabled_colors_are_all_empty() {
        let c = Colors::new(false, None);
        assert!(c.red.is_empty(), "red should be empty when disabled");
        assert!(c.green.is_empty(), "green should be empty when disabled");
        assert!(c.yellow.is_empty(), "yellow should be empty when disabled");
        assert!(c.cyan.is_empty(), "cyan should be empty when disabled");
        assert!(c.bold.is_empty(), "bold should be empty when disabled");
        assert!(c.reset.is_empty(), "reset should be empty when disabled");
        assert!(
            c.service_key.is_empty(),
            "service_key should be empty when disabled"
        );
    }

    // Colors::new(true, Some(...)) tests

    /// `custom_key_color_produces_truecolor_escape()` asserts that a valid hex color is converted
    /// to the correct 24-bit ANSI foreground escape sequence
    ///
    #[test]
    fn custom_key_color_produces_truecolor_escape() {
        let c = Colors::new(true, Some("#ff8800"));
        // #ff8800 → R=255, G=136, B=0
        assert_eq!(c.service_key, "\x1b[38;2;255;136;0m");
    }

    /// `custom_key_color_uppercase_hex_is_accepted()` asserts that uppercase hex digits are
    /// normalized correctly
    ///
    #[test]
    fn custom_key_color_uppercase_hex_is_accepted() {
        let c = Colors::new(true, Some("#FF8800"));
        assert_eq!(c.service_key, "\x1b[38;2;255;136;0m");
    }

    /// `custom_key_color_disabled_remains_empty()` asserts that a hex color has no effect when
    /// color output is disabled
    ///
    #[test]
    fn custom_key_color_disabled_remains_empty() {
        let c = Colors::new(false, Some("#ff8800"));
        assert!(
            c.service_key.is_empty(),
            "service_key must be empty when color is disabled"
        );
    }

    /// `default_key_color_matches_cyan()` asserts that the default service key color is cyan
    ///
    #[test]
    fn default_key_color_matches_cyan() {
        use crate::constants::CYAN;
        let c = Colors::new(true, None);
        assert_eq!(c.service_key, CYAN);
    }
}
