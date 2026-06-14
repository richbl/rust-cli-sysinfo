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
