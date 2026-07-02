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
    /// then sorts entries by their declared [`ServiceMeta::order`] for a default display order
    /// (tie-breakers are resolved by the `build.rs`'s discovery order)
    ///
    #[must_use]
    pub fn new(ctx: &ServiceContext) -> Self {
        let mut entries = services::build_all(ctx);
        entries.sort_by_key(|(meta, _)| meta.sort_order);

        let registry = Self { entries };
        registry.debug_validate();
        registry
    }

    /// `from_entries()` is a test-only constructor allowing registry behavior (token lookup,
    /// duplicate/empty-token detection, ordering) to be tested without touching real system
    /// files via the real services
    ///
    #[cfg(test)]
    pub(crate) fn from_entries(mut entries: Vec<(ServiceMeta, Box<dyn ErasedService>)>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::ServiceContext;
    use crate::presentation::colors::Colors;
    use std::any::Any;
    use std::path::PathBuf;

    /// Minimal `ErasedService` stand-in used to test registry behavior without touching real
    /// system files.
    struct DummyService;

    impl ErasedService for DummyService {
        /// `collect_erased()` returns `Ok(Box::new(()))`
        ///
        fn collect_erased(&self) -> crate::core::erased::CollectResult {
            Ok(Box::new(()))
        }

        /// `render_erased()` sets a dummy test service and returns `Ok(())`
        ///
        fn render_erased(
            &self,
            _label: &str,
            _data: &(dyn Any + Send + Sync),
            _colors: &Colors,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    use crate::core::error::AppError;

    /// `meta()` builds a `ServiceMeta` for a dummy test service
    ///
    fn meta(token: &'static str, sort_order: u16) -> ServiceMeta {
        ServiceMeta {
            token,
            label: "  Dummy",
            description: "a dummy service for tests",
            sort_order,
        }
    }

    // ServiceRegistry::len()/is_empty() tests

    /// `len_matches_entry_count()` asserts that `len()` reflects the number of entries provided
    ///
    #[test]
    fn len_matches_entry_count() {
        let registry = ServiceRegistry::from_entries(vec![
            (meta("A", 0), Box::new(DummyService)),
            (meta("B", 1), Box::new(DummyService)),
        ]);
        assert_eq!(registry.len(), 2); // is_empty() assertion removed along with the method
    }

    // ServiceRegistry::index_of() tests

    /// `index_of_finds_exact_token()` asserts that a known token resolves to its index
    ///
    #[test]
    fn index_of_finds_exact_token() {
        let registry = ServiceRegistry::from_entries(vec![
            (meta("A", 0), Box::new(DummyService)),
            (meta("B", 1), Box::new(DummyService)),
        ]);
        assert_eq!(registry.index_of("B"), Some(1));
    }

    /// `index_of_is_case_insensitive()` asserts that token lookup ignores case
    ///
    #[test]
    fn index_of_is_case_insensitive() {
        let registry =
            ServiceRegistry::from_entries(vec![(meta("CPU", 0), Box::new(DummyService))]);
        assert_eq!(registry.index_of("cpu"), Some(0));
        assert_eq!(registry.index_of("Cpu"), Some(0));
    }

    /// `index_of_unknown_token_returns_none()` asserts that an unknown token resolves to `None`
    ///
    #[test]
    fn index_of_unknown_token_returns_none() {
        let registry = ServiceRegistry::from_entries(vec![(meta("A", 0), Box::new(DummyService))]);
        assert_eq!(registry.index_of("ZZZ"), None);
    }

    // ServiceRegistry ordering test

    /// `entries_are_sorted_by_declared_order()` asserts that entries are reordered by
    /// `ServiceMeta::order` regardless of the order they were constructed
    ///
    #[test]
    fn entries_are_sorted_by_declared_order() {
        let registry = ServiceRegistry::from_entries(vec![
            (meta("SECOND", 5), Box::new(DummyService)),
            (meta("FIRST", 1), Box::new(DummyService)),
        ]);
        let tokens: Vec<&str> = registry.all_meta().map(|m| m.token).collect();
        assert_eq!(tokens, vec!["FIRST", "SECOND"]);
    }

    // ServiceRegistry::debug_validate() tests

    /// `duplicate_token_panics_in_debug()` asserts that constructing a registry with duplicate
    /// tokens panics
    ///
    #[test]
    #[should_panic(expected = "duplicate service token")]
    fn duplicate_token_panics_in_debug() {
        let _ = ServiceRegistry::from_entries(vec![
            (meta("DUP", 0), Box::new(DummyService)),
            (meta("DUP", 1), Box::new(DummyService)),
        ]);
    }

    /// `empty_token_panics_in_debug()` asserts that constructing a registry with an empty token
    /// panics
    ///
    #[test]
    #[should_panic(expected = "must not have an empty token")]
    fn empty_token_panics_in_debug() {
        let _ = ServiceRegistry::from_entries(vec![(meta("", 0), Box::new(DummyService))]);
    }

    /// `real_registry_has_no_duplicate_or_empty_tokens()` exercises the actual construction path
    /// `main()` uses, so a real regression (e.g. two service files claiming the same token) is
    /// caught by `cargo test` instead of only surfacing when someone runs the binary
    ///
    #[test]
    fn real_registry_has_no_duplicate_or_empty_tokens() {
        let ctx = ServiceContext {
            disk_mount: PathBuf::from("/"),
            cpu_sample_ms: 1,
        };
        let registry = ServiceRegistry::new(&ctx); // panics on duplicate/empty tokens via debug_validate
        assert!(registry.len() > 0);
    }
}
