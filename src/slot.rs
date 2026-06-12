/// `SlotFilter` represents the three operating modes of the `-s` flag
pub enum SlotFilter {
    Default,                  // No `-s` flag: render the full default utility output
    ShowLabeled,              // `-s` with no argument: render labeled output and exit
    Custom(Vec<ServiceSlot>), // `-s TOKENS`: render only the specified slots in the given order
}

/// `ServiceSlot` identifies each service row in the output
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceSlot {
    Os,
    Hst,
    Cpu,
    Gpu,
    Knl,
    Upt,
    Load,
    CpuU,
    RamU,
    DskU,
    Usr,
}

/// `ServiceSlot` methods
impl ServiceSlot {
    /// `token()` returns the token string for this slot (used in `-s` output and parsing)
    ///
    pub const fn token(self) -> &'static str {
        match self {
            Self::Os => "OS",
            Self::Hst => "HST",
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
            Self::Knl => "KNL",
            Self::Upt => "UPT",
            Self::Load => "LOAD",
            Self::CpuU => "CPUU",
            Self::RamU => "RAMU",
            Self::DskU => "DSKU",
            Self::Usr => "USR",
        }
    }

    /// `description()` returns a human-readable description shown next to the token in `-s` labeled
    /// output
    ///
    pub const fn description(self) -> &'static str {
        match self {
            Self::Os => "Operating system name and version",
            Self::Hst => "System hostname",
            Self::Cpu => "CPU model",
            Self::Gpu => "GPU model(s)",
            Self::Knl => "Linux kernel version",
            Self::Upt => "System uptime",
            Self::Load => "Load averages (1m, 5m, 15m)",
            Self::CpuU => "CPU usage %",
            Self::RamU => "Memory usage % (Used/Total)",
            Self::DskU => "Disk usage % (Used/Total)",
            Self::Usr => "Current users",
        }
    }

    /// The default ordered list of all slots, defining the standard output layout
    pub const ALL: &'static [Self] = &[
        Self::Os,
        Self::Hst,
        Self::Cpu,
        Self::Gpu,
        Self::Knl,
        Self::Upt,
        Self::Load,
        Self::CpuU,
        Self::RamU,
        Self::DskU,
        Self::Usr,
    ];

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
        match token {
            "OS" => Some(Self::Os),
            "HST" => Some(Self::Hst),
            "CPU" => Some(Self::Cpu),
            "GPU" => Some(Self::Gpu),
            "KNL" => Some(Self::Knl),
            "UPT" => Some(Self::Upt),
            "LOAD" => Some(Self::Load),
            "CPUU" => Some(Self::CpuU),
            "RAMU" => Some(Self::RamU),
            "DSKU" => Some(Self::DskU),
            "USR" => Some(Self::Usr),
            _ => None,
        }
    }
}
