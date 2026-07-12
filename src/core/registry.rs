use crate::core::context::ServiceContext;
use crate::core::erased::ErasedService;
use crate::core::meta::ServiceMeta;
use crate::services;

/// `ServiceRegistry` is the runtime container for every service discovered by `build.rs` in
/// `src/services`
///
pub struct ServiceRegistry {
    entries: Vec<(ServiceMeta, Box<dyn ErasedService>)>,
}

impl ServiceRegistry {
    /// `new()` builds one entry per service discovered under `src/services` at compile time,
    /// then sorts entries by their declared [`ServiceMeta::sort_order`] for a default display order
    /// (tie-breakers are resolved by the `build.rs`'s discovery order)
    ///
    #[must_use]
    pub fn new(ctx: &ServiceContext) -> Self {
        let entries = services::build_all(ctx);
        Self::build(entries)
    }

    /// `from_entries()` is a test-only constructor allowing registry behavior (token lookup,
    /// duplicate/empty-token detection, ordering) to be tested without touching real system
    /// files via the real services
    ///
    #[cfg(test)]
    pub(crate) fn from_entries(entries: Vec<(ServiceMeta, Box<dyn ErasedService>)>) -> Self {
        Self::build(entries)
    }

    /// `build()` sorts `entries` by their declared [`ServiceMeta::sort_order`], wraps them in a
    /// `ServiceRegistry`, and runs [`Self::debug_validate`] against the result
    ///
    fn build(mut entries: Vec<(ServiceMeta, Box<dyn ErasedService>)>) -> Self {
        entries.sort_by_key(|(meta, _)| meta.sort_order);
        let registry = Self { entries };
        registry.debug_validate();
        registry
    }

    /// `len()` returns the number of discovered services
    ///
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `meta()` returns the metadata for the service at `idx`
    ///
    #[must_use]
    pub fn meta(&self, idx: usize) -> &ServiceMeta {
        &self.entries[idx].0
    }

    /// `service()` returns the collectible/renderable service at `idx`
    ///
    #[must_use]
    pub fn service(&self, idx: usize) -> &dyn ErasedService {
        &*self.entries[idx].1
    }

    /// `all_meta()` iterates the metadata of every discovered service, in display order
    ///
    pub fn all_meta(&self) -> impl Iterator<Item = &ServiceMeta> {
        self.entries.iter().map(|(meta, _)| meta)
    }

    /// `index_of()` looks up a service's index by its token (case-insensitive)
    ///
    #[must_use]
    pub fn index_of(&self, token: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|(meta, _)| meta.token.eq_ignore_ascii_case(token))
    }

    /// `debug_validate()` guards against a service file introducing an empty or duplicate
    /// service token
    ///
    fn debug_validate(&self) {
        #[cfg(debug_assertions)]
        {
            let mut seen = std::collections::HashSet::new();
            for (meta, _) in &self.entries {
                assert!(
                    !meta.token.is_empty(),
                    "a service must not have an empty token"
                );
                assert!(
                    seen.insert(meta.token),
                    "duplicate service token '{}': every service must have a unique token",
                    meta.token
                );
            }
        }
    }
}
