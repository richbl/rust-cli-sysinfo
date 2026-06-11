/// `SlotFilter` represents the three operating modes of the `-s` flag
pub enum SlotFilter {
    /// No `-s` flag: render the full default utility output
    Default,
    /// `-s` with no argument: render labeled output and exit
    ShowLabeled,
    /// `-s TOKENS`: render only the specified slots in the given order
    Custom(Vec<ServiceSlot>),
}

/// `ServiceSlot` identifies each service row in the output
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceSlot {
    Os,
    Hst,
    CpuM,
    Gpu,
    Knl,
    Upt,
    Load,
    Cpu,
    Ram,
    Dsk,
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
            Self::CpuM => "CPUM",
            Self::Gpu => "GPU",
            Self::Knl => "KNL",
            Self::Upt => "UPT",
            Self::Load => "LOAD",
            Self::Cpu => "CPU",
            Self::Ram => "RAM",
            Self::Dsk => "DSK",
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
            Self::CpuM => "CPU model",
            Self::Gpu => "GPU model(s)",
            Self::Knl => "Linux kernel version",
            Self::Upt => "System uptime",
            Self::Load => "Load averages (1m, 5m, 15m)",
            Self::Cpu => "CPU usage %",
            Self::Ram => "Memory usage % (Used/Total)",
            Self::Dsk => "Disk usage % (Used/Total)",
            Self::Usr => "Current users",
        }
    }

    /// The default ordered list of all slots, defining the standard output layout
    pub const ALL: &'static [Self] = &[
        Self::Os,
        Self::Hst,
        Self::CpuM,
        Self::Gpu,
        Self::Knl,
        Self::Upt,
        Self::Load,
        Self::Cpu,
        Self::Ram,
        Self::Dsk,
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
                    .ok_or_else(|| format!("unknown service token '{token}' (run `-s` with no argument to see available tokens)"))
            })
            .collect()
    }

    /// `from_token()` looks up a `ServiceSlot` by its token string (case-insensitive)
    ///
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "OS" => Some(Self::Os),
            "HST" => Some(Self::Hst),
            "CPUM" => Some(Self::CpuM),
            "GPU" => Some(Self::Gpu),
            "KNL" => Some(Self::Knl),
            "UPT" => Some(Self::Upt),
            "LOAD" => Some(Self::Load),
            "CPU" => Some(Self::Cpu),
            "RAM" => Some(Self::Ram),
            "DSK" => Some(Self::Dsk),
            "USR" => Some(Self::Usr),
            _ => None,
        }
    }
}
