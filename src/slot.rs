/// `SlotFilter` represents the three operating modes of the `-s` flag
pub enum SlotFilter {
    Default,                  // No `-s` flag: render the full default utility output
    ShowLabeled,              // `-s` with no argument: render labeled output and exit
    Custom(Vec<ServiceSlot>), // `-s TOKENS`: render only the specified slots in the given order
}

/// `SlotMeta` bundles a slot's `-s` token and its human-readable description
///
struct SlotMeta {
    slot: ServiceSlot,
    token: &'static str,
    description: &'static str,
}

/// `define_slots!` generates the `SLOT_TABLE` and `ServiceSlot` enum
macro_rules! define_slots {
    ($($variant:ident: $token:literal, $description:literal);* $(;)?) => {

        /// `ServiceSlot` identifies each service row in the output
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum ServiceSlot {
            $($variant),*
        }

        /// The ordered table of all known slots
        const SLOT_TABLE: &[SlotMeta] = &[
            $(
                SlotMeta {
                    slot: ServiceSlot::$variant,
                    token: $token,
                    description: $description,
                },
            )*
        ];
    }
}

// `define_slots!` macro generates the `SLOT_TABLE` and `ServiceSlot` enum
define_slots! {
    Os:   "OS",   "Operating system name and version";
    Hst:  "HST",  "System hostname";
    Cpu:  "CPU",  "CPU model";
    Gpu:  "GPU",  "GPU model(s)";
    Knl:  "KNL",  "Linux kernel version";
    Upt:  "UPT",  "System uptime";
    Load: "LOAD", "Load averages (1m, 5m, 15m)";
    CpuU: "CPUU", "CPU usage %";
    RamU: "RAMU", "Memory usage % (Used/Total)";
    DskU: "DSKU", "Disk usage % (Used/Total)";
    Usr:  "USR",  "Current users";
}

/// `ServiceSlot` methods
impl ServiceSlot {
    /// `meta()` looks up this slot's entry in `SLOT_TABLE`
    ///
    fn meta(self) -> &'static SlotMeta {
        SLOT_TABLE
            .iter()
            .find(|m| m.slot == self)
            .expect("every ServiceSlot variant must have a SLOT_TABLE entry")
    }

    /// `token()` returns the token string for this slot (used in `-s` output and parsing)
    ///
    pub fn token(self) -> &'static str {
        self.meta().token
    }

    /// `description()` returns a description shown next to the token in `-s` labeled output
    ///
    pub fn description(self) -> &'static str {
        self.meta().description
    }

    /// `all()` returns the default ordered list of all slots, defining the standard output layout
    ///
    pub fn all() -> Vec<Self> {
        SLOT_TABLE.iter().map(|m| m.slot).collect()
    }

    /// `parse_list()` parses a hyphen-separated token string (e.g. `"HST-OS-KNL"`) into an
    /// ordered `Vec<ServiceSlot>`
    ///
    pub fn parse_list(input: &str) -> Result<Vec<Self>, String> {
        input
            .split('-')
            .map(|raw| {
                let token = raw.trim().to_uppercase();
                Self::from_token(&token)
                    .ok_or_else(|| format!("Unknown service token '{token}' (run `-s` with no argument to see available service tokens)"))
            })
            .collect()
    }

    /// `from_token()` looks up a `ServiceSlot` by its token string (case-insensitive)
    ///
    fn from_token(token: &str) -> Option<Self> {
        SLOT_TABLE.iter().find(|m| m.token == token).map(|m| m.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ServiceSlot::all() test

    #[test]
    /// `all_returns_every_defined_slot()` asserts that all defined slots are returned by `all()`
    ///
    fn all_returns_every_defined_slot() {
        assert_eq!(ServiceSlot::all().len(), 11);
    }

    #[test]
    /// `all_first_slot_is_os()` asserts that the first slot in the default list is `Os`
    ///
    fn all_first_slot_is_os() {
        assert_eq!(ServiceSlot::all()[0], ServiceSlot::Os);
    }

    #[test]
    /// `all_last_slot_is_usr()` asserts that the last slot in the default list is `Usr`
    ///
    fn all_last_slot_is_usr() {
        assert_eq!(
            ServiceSlot::all().last().copied().unwrap(),
            ServiceSlot::Usr
        );
    }

    #[test]
    /// `all_contains_no_duplicates()` asserts that the slot list contains no duplicates
    ///
    fn all_contains_no_duplicates() {
        let slots = ServiceSlot::all();
        let mut seen = std::collections::HashSet::new();
        for slot in &slots {
            assert!(seen.insert(slot), "duplicate slot found: {slot:?}");
        }
    }

    // ServiceSlot::token() test

    #[test]
    /// `token_os_is_uppercase_os()` asserts that the token for the `Os` slot is "OS"
    ///
    fn token_os_is_uppercase_os() {
        assert_eq!(ServiceSlot::Os.token(), "OS");
    }

    #[test]
    /// `token_hst_is_uppercase_hst()` asserts that the token for the `Hst` slot is "HST"
    ///
    fn token_hst_is_uppercase_hst() {
        assert_eq!(ServiceSlot::Hst.token(), "HST");
    }

    #[test]
    /// `token_cpuu_has_double_u()` asserts that the token for the `CpuU` slot is "CPUU"
    ///
    fn token_cpuu_has_double_u() {
        // Regression guard: "CPUU" is easy to mistype as "CPU" when adding new slots
        assert_eq!(ServiceSlot::CpuU.token(), "CPUU");
    }

    #[test]
    /// `all_tokens_are_non_empty()` asserts that all slot tokens are non-empty
    ///
    fn all_tokens_are_non_empty() {
        for slot in ServiceSlot::all() {
            assert!(!slot.token().is_empty(), "{slot:?} has an empty token");
        }
    }

    #[test]
    /// `all_tokens_are_uppercase()` asserts that all slot tokens are uppercase
    ///
    fn all_tokens_are_uppercase() {
        for slot in ServiceSlot::all() {
            let token = slot.token();
            assert_eq!(
                token,
                token.to_uppercase(),
                "{slot:?} token is not uppercase"
            );
        }
    }

    // ServiceSlot::description() test

    #[test]
    /// `all_descriptions_are_non_empty()` asserts that all slot descriptions are non-empty
    ///
    fn all_descriptions_are_non_empty() {
        for slot in ServiceSlot::all() {
            assert!(
                !slot.description().is_empty(),
                "{slot:?} has an empty description"
            );
        }
    }

    // ServiceSlot::parse_list() test

    #[test]
    /// `parse_list_single_known_token()` asserts that parsing a single known token succeeds
    ///
    fn parse_list_single_known_token() {
        let result = ServiceSlot::parse_list("OS");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![ServiceSlot::Os]);
    }

    #[test]
    /// `parse_list_multiple_tokens_preserves_order()` asserts that parsing multiple tokens
    /// preserves their order
    ///
    fn parse_list_multiple_tokens_preserves_order() {
        let result = ServiceSlot::parse_list("OS-CPU-GPU");
        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(
            slots,
            vec![ServiceSlot::Os, ServiceSlot::Cpu, ServiceSlot::Gpu]
        );
    }

    #[test]
    /// `parse_list_is_case_insensitive()` asserts that token parsing is case-insensitive
    ///
    fn parse_list_is_case_insensitive() {
        let lower = ServiceSlot::parse_list("os-cpu");
        let upper = ServiceSlot::parse_list("OS-CPU");
        let mixed = ServiceSlot::parse_list("Os-Cpu");
        assert_eq!(lower.unwrap(), upper.as_ref().unwrap().clone());
        assert_eq!(mixed.unwrap(), upper.unwrap());
    }

    #[test]
    /// `parse_list_all_tokens_round_trips()` asserts that parsing the joined string of all tokens
    /// round-trips correctly
    ///
    fn parse_list_all_tokens_round_trips() {
        // Build a hyphen-joined string from the canonical all() order and
        // verify it round-trips back to the same vec
        let canonical = ServiceSlot::all();
        let joined: String = canonical
            .iter()
            .map(|s| s.token())
            .collect::<Vec<_>>()
            .join("-");
        let parsed = ServiceSlot::parse_list(&joined).expect("all tokens must parse");
        assert_eq!(parsed, canonical);
    }

    #[test]
    /// `parse_list_unknown_token_returns_err_containing_token()` asserts that parsing an unknown
    /// token returns an error containing that token
    ///
    fn parse_list_unknown_token_returns_err_containing_token() {
        let result = ServiceSlot::parse_list("INVALID");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("INVALID"));
    }

    #[test]
    /// `parse_list_partially_invalid_fails_on_bad_token()` asserts that parsing a list containing
    /// an invalid token fails and references the bad token
    ///
    fn parse_list_partially_invalid_fails_on_bad_token() {
        let result = ServiceSlot::parse_list("OS-BOGUS-CPU");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BOGUS"));
    }

    #[test]
    /// `parse_list_empty_string_returns_err()` asserts that parsing an empty string returns an
    /// error
    ///
    fn parse_list_empty_string_returns_err() {
        let result = ServiceSlot::parse_list("");
        assert!(result.is_err(), "empty input must not silently succeed");
    }

    #[test]
    /// `parse_list_whitespace_only_returns_err()` asserts that parsing whitespace-only input
    /// returns an error
    ///
    fn parse_list_whitespace_only_returns_err() {
        let result = ServiceSlot::parse_list("   ");
        assert!(result.is_err());
    }

    #[test]
    /// `parse_list_duplicate_tokens_are_currently_allowed()` asserts that duplicate tokens are
    /// permitted in parsing
    ///
    fn parse_list_duplicate_tokens_are_currently_allowed() {
        let result = ServiceSlot::parse_list("OS-OS-CPU");
        assert!(result.is_ok());
        let slots = result.unwrap();
        assert_eq!(slots.len(), 3);
        assert_eq!(slots[0], ServiceSlot::Os);
        assert_eq!(slots[1], ServiceSlot::Os);
    }
}
