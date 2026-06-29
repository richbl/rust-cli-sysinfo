use std::path::PathBuf;

use crate::cli::Opts;

/// `ServiceContext` encapsulates global configuration provided to all services
/// during initialization.
pub struct ServiceContext {
    pub disk_mount: PathBuf,
    pub cpu_sample_ms: u64,
}

impl From<&Opts> for ServiceContext {
    fn from(opts: &Opts) -> Self {
        Self {
            disk_mount: PathBuf::from(&opts.disk_mount),
            cpu_sample_ms: opts.cpu_sample_ms,
        }
    }
}
