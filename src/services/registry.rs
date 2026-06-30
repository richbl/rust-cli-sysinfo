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

/// `ServiceRegistry` acts as a factory and container for all known services
pub struct ServiceRegistry {
    os: AnyService,
    hst: AnyService,
    cpu: AnyService,
    gpu: AnyService,
    knl: AnyService,
    upt: AnyService,
    load: AnyService,
    cpu_u: AnyService,
    ram_u: AnyService,
    dsk_u: AnyService,
    usr: AnyService,
}

impl ServiceRegistry {
    /// `new()` constructs one `AnyService` for every known service, wiring in configuration
    /// from the `ServiceContext`
    ///
    pub fn new(ctx: &ServiceContext) -> Self {
        Self {
            os: AnyService::Os(OsService),
            hst: AnyService::Hostname(HostnameService),
            cpu: AnyService::CpuModel(CpuModelService),
            gpu: AnyService::Gpu(GpuService),
            knl: AnyService::Kernel(KernelService),
            upt: AnyService::Uptime(UptimeService),
            load: AnyService::LoadAvg(LoadAvgService),
            cpu_u: AnyService::CpuUsage(CpuUsageService::new(ctx)),
            ram_u: AnyService::Memory(MemoryService),
            dsk_u: AnyService::Disk(DiskService::new(ctx)),
            usr: AnyService::Users(UsersService),
        }
    }

    /// `get()` retrieves the registered service for a given slot using direct pattern matching
    ///
    pub fn get(&self, slot: ServiceSlot) -> &AnyService {
        match slot {
            ServiceSlot::Os => &self.os,
            ServiceSlot::Hst => &self.hst,
            ServiceSlot::Cpu => &self.cpu,
            ServiceSlot::Gpu => &self.gpu,
            ServiceSlot::Knl => &self.knl,
            ServiceSlot::Upt => &self.upt,
            ServiceSlot::Load => &self.load,
            ServiceSlot::CpuU => &self.cpu_u,
            ServiceSlot::RamU => &self.ram_u,
            ServiceSlot::DskU => &self.dsk_u,
            ServiceSlot::Usr => &self.usr,
        }
    }
}
