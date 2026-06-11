use std::fs::File;
use std::io::{BufRead, BufReader};

use super::prelude::*;

/// `CpuModelInfo` parsed from `/proc/cpuinfo`
#[derive(Default)]
pub struct CpuModelInfo {
    pub model: Option<String>,
}

/// `CpuModelService` is a service for collecting and rendering the CPU model name
pub struct CpuModelService;

/// `CpuModelService` implements the `Service` trait
impl Service for CpuModelService {
    type Data = CpuModelInfo;

    /// `collect()` reads the first `model name` entry from `/proc/cpuinfo`
    ///
    fn collect(&self) -> Result<Self::Data, AppError> {
        Ok(CpuModelInfo {
            model: read_cpu_model(),
        })
    }

    /// `render()` renders the CPU model name
    ///
    fn render(&self, data: &Self::Data, c: &Colors) {
        print_row(
            "  CPU:",
            data.model.as_deref().unwrap_or("Unknown"),
            &Threshold::None,
            c,
        );
    }
}

/// `read_cpu_model()` reads the CPU model name from the first `model name` entry in
/// `/proc/cpuinfo`, returning on first match for efficiency
///
fn read_cpu_model() -> Option<String> {
    let file = File::open("/proc/cpuinfo").ok()?;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("model name")
            && let Some((_, val)) = line.split_once(':')
        {
            return Some(val.trim().to_string());
        }
    }

    None
}
