pub mod cpu_model;
pub mod cpu_usage;
pub mod disk;
pub mod gpu;
pub mod hostname;
pub mod kernel;
pub mod load_avg;
pub mod memory;
pub mod os_name;
pub mod registry;
pub mod uptime;
pub mod users;

use crate::core::error::AppError;
use crate::presentation::colors::Colors;

use cpu_model::{CpuModelInfo, CpuModelService};
use cpu_usage::{CpuUsageInfo, CpuUsageService};
use disk::{DiskInfo, DiskService};
use gpu::{GpuInfo, GpuService};
use hostname::{HostnameInfo, HostnameService};
use kernel::{KernelInfo, KernelService};
use load_avg::{LoadAvgInfo, LoadAvgService};
use memory::{MemInfo, MemoryService};
use os_name::{OsInfo, OsService};
use uptime::{UptimeInfo, UptimeService};
use users::{UsersInfo, UsersService};

/// `prelude` re-exports the types shared across all service modules, eliminating
/// duplicate `use` statements in each service file
pub mod prelude {
    pub use super::Service;
    pub use crate::core::error::AppError;
    pub use crate::presentation::colors::Colors;
    pub use crate::presentation::format::{Threshold, print_row};
}

/// `Service` is the common interface implemented by every system information service
pub trait Service {
    type Data;

    /// `collect()` reads raw system data and returns it
    ///
    fn collect(&self) -> Result<Self::Data, AppError>;

    /// `render()` formats and prints `data` to stdout, propagating any formatting or output errors
    ///
    fn render(&self, label: &str, data: &Self::Data, colors: &Colors) -> Result<(), AppError>;
}

/// `ServiceData` wraps the concrete `Data` struct for every known service
pub enum ServiceData {
    Os(OsInfo),
    Hostname(HostnameInfo),
    CpuModel(CpuModelInfo),
    Gpu(GpuInfo),
    Kernel(KernelInfo),
    Uptime(UptimeInfo),
    LoadAvg(LoadAvgInfo),
    CpuUsage(CpuUsageInfo),
    Memory(MemInfo),
    Disk(DiskInfo),
    Users(UsersInfo),
}

/// `AnyService` wraps the concrete service implementations
pub enum AnyService {
    Os(OsService),
    Hostname(HostnameService),
    CpuModel(CpuModelService),
    Gpu(GpuService),
    Kernel(KernelService),
    Uptime(UptimeService),
    LoadAvg(LoadAvgService),
    CpuUsage(CpuUsageService),
    Memory(MemoryService),
    Disk(DiskService),
    Users(UsersService),
}

/// `dispatch_render!` dispatches rendering to the underlying concrete service
macro_rules! dispatch_render {
    ($self:expr, $data:expr, $label:expr, $colors:expr, { $($variant:ident => $data_variant:path),* $(,)? }) => {
        match ($self, $data) {
            $(
                (Self::$variant(s), $data_variant(d)) => {
                    // Directly propagate the Result<(), AppError> from the service's render call
                    s.render($label, d, $colors)
                }
            )*
            _ => {
                // Fail-fast on developer implementation bugs
                panic!(
                    "Error: Service and Data type mismatch",
                );
            }
        }
    };
}

impl AnyService {
    /// `collect()` dispatches to the underlying concrete service and maps the result into the
    /// corresponding `ServiceData` variant
    ///
    pub fn collect(&self) -> Result<ServiceData, AppError> {
        match self {
            Self::Os(s) => s.collect().map(ServiceData::Os),
            Self::Hostname(s) => s.collect().map(ServiceData::Hostname),
            Self::CpuModel(s) => s.collect().map(ServiceData::CpuModel),
            Self::Gpu(s) => s.collect().map(ServiceData::Gpu),
            Self::Kernel(s) => s.collect().map(ServiceData::Kernel),
            Self::Uptime(s) => s.collect().map(ServiceData::Uptime),
            Self::LoadAvg(s) => s.collect().map(ServiceData::LoadAvg),
            Self::CpuUsage(s) => s.collect().map(ServiceData::CpuUsage),
            Self::Memory(s) => s.collect().map(ServiceData::Memory),
            Self::Disk(s) => s.collect().map(ServiceData::Disk),
            Self::Users(s) => s.collect().map(ServiceData::Users),
        }
    }

    /// `render()` hands off rendering to the underlying concrete service
    ///
    pub fn render(&self, label: &str, data: &ServiceData, colors: &Colors) -> Result<(), AppError> {
        dispatch_render!(self, data, label, colors, {
            Os => ServiceData::Os,
            Hostname => ServiceData::Hostname,
            CpuModel => ServiceData::CpuModel,
            Gpu => ServiceData::Gpu,
            Kernel => ServiceData::Kernel,
            Uptime => ServiceData::Uptime,
            LoadAvg => ServiceData::LoadAvg,
            CpuUsage => ServiceData::CpuUsage,
            Memory => ServiceData::Memory,
            Disk => ServiceData::Disk,
            Users => ServiceData::Users,
        })
    }
}
