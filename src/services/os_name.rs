use std::fs::File;
use std::io::{BufRead, BufReader};

use super::prelude::*;

/// `OsInfo` contains the OS name parsed from `/etc/os-release`
pub struct OsInfo {
    pub name: String,
}

/// `OsService` is a struct for collecting and rendering the OS name
pub struct OsService;

/// `OsService` implements the `Service` trait
impl Service for OsService {
    type Data = OsInfo;

    /// `collect()` reads `PRETTY_NAME` from `/etc/os-release`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        let name = read_os_name().unwrap_or_else(|| "Unknown Linux".into());
        Ok(OsInfo { name })
    }

    /// `render()` renders the OS name
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        print_row("  OS:", &data.name, &Threshold::None, c);
    }
}

/// `read_os_name()` reads the `PRETTY_NAME` field from `/etc/os-release`
///
fn read_os_name() -> Option<String> {
    let file = File::open("/etc/os-release").ok()?;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
            return Some(val.trim_matches('"').to_string());
        }
    }

    None
}
