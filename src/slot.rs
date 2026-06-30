/// `SlotFilter` represents the three operating modes of the `-s` flag
pub enum SlotFilter {
    /// No `-s` flag: render the full default utility output
    Default,
    /// `-s` with no argument: render labeled output and exit
    ShowLabeled,
    /// `-s TOKENS`: render only the specified slots in the given order
    Custom(Vec<ServiceSlot>),
}

/// `define_slots!` generates the `ServiceSlot` enum and its property accessors
macro_rules! define_slots {
    ($($variant:ident: $token:literal, $label:literal, $description:literal);* $(;)?) => {

        /// `ServiceSlot` identifies each service row in the output
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum ServiceSlot {
            $($variant),*
        }

        /// Static compilation of all available slots, keeping their default order
        const ALL_SLOTS: &[ServiceSlot] = &[
            $(ServiceSlot::$variant),*
        ];

        impl ServiceSlot {
            /// `token()` returns the token string for this slot
            ///
            #[must_use]
            pub const fn token(self) -> &'static str {
                match self {
                    $(Self::$variant => $token),*
                }
            }

            /// `label()` returns the display label string for this slot
            ///
            #[must_use]
            pub const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),*
                }
            }

            /// `description()` returns a description shown in `-s` labeled output
            ///
            #[must_use]
            pub const fn description(self) -> &'static str {
                match self {
                    $(Self::$variant => $description),*
                }
            }

            /// `all()` returns the default ordered list of all slots as a static slice, avoiding heap allocation
            ///
            #[must_use]
            pub const fn all() -> &'static [Self] {
                ALL_SLOTS
            }

            /// `from_token()` looks up a `ServiceSlot` by its token string (case-sensitive lookup)
            ///
            fn from_token(token: &str) -> Option<Self> {
                match token {
                    $($token => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    }
}

// Generates the enum and safe properties at compile time
define_slots! {
    Os:   "OS",   "  OS:",             "Operating system name and version";
    Hst:  "HST",  "  Hostname:",       "System hostname";
    Cpu:  "CPU",  "  CPU:",            "CPU model";
    Gpu:  "GPU",  "  GPU(s):",         "GPU model(s)";
    Knl:  "KNL",  "  Kernel:",         "Linux kernel version";
    Upt:  "UPT",  "  Uptime:",         "System uptime";
    Load: "LOAD", "  Load averages:",  "Load averages (1m, 5m, 15m)";
    CpuU: "CPUU", "  CPU usage:",      "CPU usage %";
    RamU: "RAMU", "  Memory usage:",   "Memory usage % (Used/Total)";
    DskU: "DSKU", "  Disk usage:",     "Disk usage % (Used/Total)";
    Usr:  "USR",  "  User(s):",         "Current users";
}

impl ServiceSlot {
    /// `parse_list()` parses a hyphen-separated token string (e.g. `"HST-OS-KNL"`) into an
    /// ordered `Vec<ServiceSlot>`
    ///
    pub fn parse_list(input: &str) -> Result<Vec<Self>, String> {
        if input.trim().is_empty() {
            return Err("service token list cannot be empty".to_string());
        }

        input
            .split('-')
            .map(|raw| {
                let token = raw.trim().to_uppercase();
                Self::from_token(&token)
                    .ok_or_else(|| format!("Unknown service token '{token}' (run `-s` with no argument to see available service tokens)"))
            })
            .collect()
    }
}

impl SlotFilter {
    /// `to_active_slots()` returns `Some(active slots)` for renderable filters, or `None` for `ShowLabeled`
    ///
    pub fn to_active_slots(&self) -> Option<Vec<ServiceSlot>> {
        match self {
            Self::Default => Some(ServiceSlot::all().to_vec()),
            Self::Custom(slots) => Some(slots.clone()),
            Self::ShowLabeled => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ServiceSlot::all() test

    /// `all_returns_every_defined_slot()` asserts that all defined slots are returned by `all()`
    ///
    #[test]
    fn all_returns_every_defined_slot() {
        assert_eq!(ServiceSlot::all().len(), 11);
    }

    /// `all_first_slot_is_os()` asserts that the first slot in the default list is `Os`
    ///
    #[test]
    fn all_first_slot_is_os() {
        assert_eq!(ServiceSlot::all()[0], ServiceSlot::Os);
    }

    /// `all_last_slot_is_usr()` asserts that the last slot in the default list is `Usr`
    ///
    #[test]
    fn all_last_slot_is_usr() {
        assert_eq!(
            ServiceSlot::all().last().copied().unwrap(),
            ServiceSlot::Usr
        );
    }

    /// `all_contains_no_duplicates()` asserts that the slot list contains no duplicates
    ///
    #[test]
    fn all_contains_no_duplicates() {
        let slots = ServiceSlot::all();
        let mut seen = std::collections::HashSet::new();
        for slot in slots {
            assert!(seen.insert(slot), "duplicate slot found: {slot:?}");
        }
    }

    // ServiceSlot::token() test

    /// `token_os_is_uppercase_os()` asserts that the token for the `Os` slot is "OS"
    ///
    #[test]
    fn token_os_is_uppercase_os() {
        assert_eq!(ServiceSlot::Os.token(), "OS");
    }

    /// `token_hst_is_uppercase_hst()` asserts that the token for the `Hst` slot is "HST"
    ///
    #[test]
    fn token_hst_is_uppercase_hst() {
        assert_eq!(ServiceSlot::Hst.token(), "HST");
    }

    /// `token_cpuu_has_double_u()` asserts that the token for the `CpuU` slot is "CPUU"
    ///
    #[test]
    fn token_cpuu_has_double_u() {
        assert_eq!(ServiceSlot::CpuU.token(), "CPUU");
    }

    /// `all_tokens_are_non_empty()` asserts that all slot tokens are non-empty
    ///
    #[test]
    fn all_tokens_are_non_empty() {
        for &slot in ServiceSlot::all() {
            assert!(!slot.token().is_empty(), "{slot:?} has an empty token");
        }
    }

    /// `all_tokens_are_uppercase()` asserts that all slot tokens are uppercase
    ///
    #[test]
    fn all_tokens_are_uppercase() {
        for &slot in ServiceSlot::all() {
            let token = slot.token();
            assert_eq!(
                token,
                token.to_uppercase(),
                "{slot:?} token is not uppercase"
            );
        }
    }

    // ServiceSlot::description() test

    /// `all_descriptions_are_non_empty()` asserts that all slot descriptions are non-empty
    ///
    #[test]
    fn all_descriptions_are_non_empty() {
        for &slot in ServiceSlot::all() {
            assert!(
                !slot.description().is_empty(),
                "{slot:?} has an empty description"
            );
        }
    }

    // ServiceSlot::label() test

    /// `all_labels_are_non_empty()` asserts that all slot labels are non-empty
    ///
    #[test]
    fn all_labels_are_non_empty() {
        for &slot in ServiceSlot::all() {
            assert!(!slot.label().is_empty(), "{slot:?} has an empty label");
        }
    }

    /// `all_labels_start_with_spaces_and_end_with_colon()` asserts that all slot labels have two
    /// leading spaces and a trailing colon
    ///
    #[test]
    fn all_labels_start_with_spaces_and_end_with_colon() {
        for &slot in ServiceSlot::all() {
            let label = slot.label();
            assert!(
                label.starts_with("  "),
                "{slot:?} label does not start with spaces: {label:?}"
            );
            assert!(
                label.ends_with(':'),
                "{slot:?} label does not end with colon: {label:?}"
            );
        }
    }

    /// `label_dsku_is_disk_usage()` asserts that the label for the `DskU` slot is correct
    ///
    #[test]
    fn label_dsku_is_disk_usage() {
        assert_eq!(ServiceSlot::DskU.label(), "  Disk usage:");
    }

    // ServiceSlot::parse_list() test

    /// `parse_list_single_known_token()` asserts that parsing a single known token succeeds
    ///
    #[test]
    fn parse_list_single_known_token() {
        let result = ServiceSlot::parse_list("OS");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![ServiceSlot::Os]);
    }

    /// `parse_list_multiple_tokens_preserves_order()` asserts that parsing multiple tokens
    /// preserves their order
    ///
    #[test]
    fn parse_list_multiple_tokens_preserves_order() {
        let result = ServiceSlot::parse_list("OS-CPU-GPU");
        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(
            slots,
            vec![ServiceSlot::Os, ServiceSlot::Cpu, ServiceSlot::Gpu]
        );
    }

    /// `parse_list_is_case_insensitive()` asserts that token parsing is case-insensitive
    ///
    #[test]
    fn parse_list_is_case_insensitive() {
        let lower = ServiceSlot::parse_list("os-cpu");
        let upper = ServiceSlot::parse_list("OS-CPU");
        let mixed = ServiceSlot::parse_list("Os-Cpu");
        assert_eq!(lower.unwrap(), upper.as_ref().unwrap().clone());
        assert_eq!(mixed.unwrap(), upper.unwrap());
    }

    /// `parse_list_all_tokens_round_trips()` asserts that parsing the joined string of all tokens
    /// round-trips correctly
    ///
    #[test]
    fn parse_list_all_tokens_round_trips() {
        let canonical = ServiceSlot::all();
        let joined: String = canonical
            .iter()
            .map(|s| s.token())
            .collect::<Vec<_>>()
            .join("-");
        let parsed = ServiceSlot::parse_list(&joined).expect("all tokens must parse");
        assert_eq!(parsed, canonical);
    }

    /// `parse_list_unknown_token_returns_err_containing_token()` asserts that parsing an unknown
    /// token returns an error containing that token
    ///
    #[test]
    fn parse_list_unknown_token_returns_err_containing_token() {
        let result = ServiceSlot::parse_list("INVALID");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("INVALID"));
    }

    /// `parse_list_partially_invalid_fails_on_bad_token()` asserts that parsing a list containing
    /// an invalid token fails and references the bad token
    ///
    #[test]
    fn parse_list_partially_invalid_fails_on_bad_token() {
        let result = ServiceSlot::parse_list("OS-BOGUS-CPU");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BOGUS"));
    }

    /// `parse_list_empty_string_returns_err()` asserts that parsing an empty string returns an
    /// error
    ///
    #[test]
    fn parse_list_empty_string_returns_err() {
        let result = ServiceSlot::parse_list("");
        assert!(result.is_err(), "empty input must not silently succeed");
        assert_eq!(result.unwrap_err(), "service token list cannot be empty");
    }

    /// `parse_list_whitespace_only_returns_err()` asserts that parsing whitespace-only input
    /// returns an error
    ///
    #[test]
    fn parse_list_whitespace_only_returns_err() {
        let result = ServiceSlot::parse_list("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "service token list cannot be empty");
    }

    /// `parse_list_duplicate_tokens_are_currently_allowed()` asserts that duplicate tokens are
    /// permitted in parsing
    ///
    #[test]
    fn parse_list_duplicate_tokens_are_currently_allowed() {
        let result = ServiceSlot::parse_list("OS-OS-CPU");
        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0], ServiceSlot::Os);
        assert_eq!(slots[1], ServiceSlot::Os);
    }
}
