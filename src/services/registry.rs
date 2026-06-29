use std::collections::HashMap;

use crate::core::context::ServiceContext;
use crate::core::error::AppError;
use crate::services::AnyService;
use crate::services::ServiceData;
use crate::services::cpu_model::CpuModelService;
use crate::services::cpu_usage::CpuUsageService;
use crate::services::disk::DiskService;
use crate::services::gpu::GpuService;
use crate::services::hostname::HostnameService;
use crate::services::kernel::KernelService;
use crate::services::load_avg::LoadAvgService;
use crate::services::memory::MemoryService;
use crate::services::os_name::OsService;
use crate::services::uptime::UptimeService;
use crate::services::users::UsersService;
use crate::slot::ServiceSlot;

/// `CollectResult` is the statically typed result of a single service's `collect()` call
pub type CollectResult = Result<ServiceData, AppError>;

/// `ServiceEntry` pairs a [`ServiceSlot`] identifier with its concrete service
/// implementation
pub struct ServiceEntry {
    pub service: AnyService,
}

/// `ServiceRegistry` acts as a factory and container for all known services.
pub struct ServiceRegistry {
    entries: HashMap<ServiceSlot, ServiceEntry>,
}

/// `impl ServiceRegistry` creates a list of [`ServiceEntry`] instances
impl ServiceRegistry {
    /// `new()` constructs one [`ServiceEntry`] for every known service, wiring in configuration
    /// from the [`ServiceContext`]
    ///
    pub fn new(ctx: &ServiceContext) -> Self {
        let entries: HashMap<_, _> = [
            (ServiceSlot::Os, AnyService::Os(OsService)),
            (ServiceSlot::Hst, AnyService::Hostname(HostnameService)),
            (ServiceSlot::Cpu, AnyService::CpuModel(CpuModelService)),
            (ServiceSlot::Gpu, AnyService::Gpu(GpuService)),
            (ServiceSlot::Knl, AnyService::Kernel(KernelService)),
            (ServiceSlot::Upt, AnyService::Uptime(UptimeService)),
            (ServiceSlot::Load, AnyService::LoadAvg(LoadAvgService)),
            (
                ServiceSlot::CpuU,
                AnyService::CpuUsage(CpuUsageService::new(ctx)),
            ),
            (ServiceSlot::RamU, AnyService::Memory(MemoryService)),
            (ServiceSlot::DskU, AnyService::Disk(DiskService::new(ctx))),
            (ServiceSlot::Usr, AnyService::Users(UsersService)),
        ]
        .into_iter()
        .map(|(slot, service)| (slot, ServiceEntry { service }))
        .collect();

        Self { entries }
    }

    /// `get()` retrieves the registered service for a given slot
    ///
    pub fn get(&self, slot: ServiceSlot) -> Option<&ServiceEntry> {
        self.entries.get(&slot)
    }
}
