use crate::core::registry::ServiceRegistry;

/// `SlotFilter` represents the three operating modes of the `-s` flag
pub enum SlotFilter {
    /// No `-s` flag: render the full default utility output, in the registry's display order.
    Default,
    /// `-s` with no argument: render the labeled reference table and exit.
    ShowLabeled,
    /// `-s TOKENS`: render only the specified services, in the given order.
    Custom(Vec<String>),
}

impl SlotFilter {
    /// `resolve()` turns this filter into concrete registry indices, or `None` for
    /// `ShowLabeled`. Unknown tokens cause the process to exit with a usage error via
    /// `on_unknown_token`
    ///
    pub fn resolve(
        &self,
        registry: &ServiceRegistry,
        on_unknown_token: impl Fn(&str, &ServiceRegistry) -> usize,
    ) -> Option<Vec<usize>> {
        match self {
            Self::Default => Some((0..registry.len()).collect()),
            Self::ShowLabeled => None,
            Self::Custom(tokens) => Some(
                tokens
                    .iter()
                    .map(|token| {
                        registry
                            .index_of(token)
                            .unwrap_or_else(|| on_unknown_token(token, registry))
                    })
                    .collect(),
            ),
        }
    }
}

/// `parse_token_list()` splits a hyphen-separated token string (e.g. `"HST-OS-KNL"`) into
/// uppercase tokens
/// This only validates *syntax* (non-empty): validity against known services is deferred to
/// [`SlotFilter::resolve`]
///
pub fn parse_token_list(input: &str) -> Result<Vec<String>, String> {
    if input.trim().is_empty() {
        return Err("service token list cannot be empty".to_string());
    }

    Ok(input
        .split('-')
        .map(|raw| raw.trim().to_uppercase())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::erased::{CollectResult, ErasedService};
    use crate::core::error::AppError;
    use crate::core::meta::ServiceMeta;
    use crate::presentation::colors::Colors;
    use std::any::Any;

    struct DummyService;

    impl ErasedService for DummyService {
        /// `collect_erased()` returns a dummy value for testing purposes
        ///
        fn collect_erased(&self) -> CollectResult {
            Ok(Box::new(()))
        }
        /// `render_erased()` is a no-op for testing purposes
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

    /// `meta()` returns a dummy `ServiceMeta` for testing purposes
    ///
    fn meta(token: &'static str, sort_order: u16) -> ServiceMeta {
        ServiceMeta {
            token,
            label: "  Dummy",
            description: "a dummy service for tests",
            sort_order,
        }
    }

    /// `dummy_registry()` returns a dummy `ServiceRegistry` for testing purposes
    ///
    fn dummy_registry() -> ServiceRegistry {
        ServiceRegistry::from_entries(vec![
            (meta("OS", 0), Box::new(DummyService)),
            (meta("CPU", 1), Box::new(DummyService)),
            (meta("GPU", 2), Box::new(DummyService)),
        ])
    }

    // parse_token_list() tests

    /// `parse_token_list_single_token()` asserts that parsing a single token succeeds
    ///
    #[test]
    fn parse_token_list_single_token() {
        assert_eq!(parse_token_list("OS").unwrap(), vec!["OS".to_string()]);
    }

    /// `parse_token_list_preserves_order()` asserts that parsing multiple tokens preserves their
    /// order
    ///
    #[test]
    fn parse_token_list_preserves_order() {
        assert_eq!(
            parse_token_list("OS-CPU-GPU").unwrap(),
            vec!["OS".to_string(), "CPU".to_string(), "GPU".to_string()]
        );
    }

    /// `parse_token_list_uppercases_tokens()` asserts that parsing lowercases/mixed-case tokens
    /// normalizes them to uppercase
    ///
    #[test]
    fn parse_token_list_uppercases_tokens() {
        assert_eq!(
            parse_token_list("os-cpu").unwrap(),
            vec!["OS".to_string(), "CPU".to_string()]
        );
    }

    /// `parse_token_list_empty_string_returns_err()` asserts that parsing an empty string
    /// returns an error
    ///
    #[test]
    fn parse_token_list_empty_string_returns_err() {
        let result = parse_token_list("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "service token list cannot be empty");
    }

    /// `parse_token_list_whitespace_only_returns_err()` asserts that parsing whitespace-only
    /// input returns an error
    ///
    #[test]
    fn parse_token_list_whitespace_only_returns_err() {
        let result = parse_token_list("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "service token list cannot be empty");
    }

    /// `parse_token_list_duplicate_tokens_are_allowed()` asserts that duplicate tokens are
    /// permitted in parsing (resolution, not parsing, is where meaning is applied)
    ///
    #[test]
    fn parse_token_list_duplicate_tokens_are_allowed() {
        let result = parse_token_list("OS-OS-CPU").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "OS");
        assert_eq!(result[1], "OS");
    }

    // SlotFilter::resolve() tests

    /// `resolve_default_returns_all_indices_in_registry_order()` asserts that `Default` resolves
    /// to every index in the registry's display order
    ///
    #[test]
    fn resolve_default_returns_all_indices_in_registry_order() {
        let registry = dummy_registry();
        let resolved = SlotFilter::Default
            .resolve(&registry, |_, _| {
                unreachable!("test bug: unknown token reached handler")
            })
            .unwrap();
        assert_eq!(resolved, vec![0, 1, 2]);
    }

    /// `resolve_show_labeled_returns_none()` asserts that `ShowLabeled` resolves to `None`
    ///
    #[test]
    fn resolve_show_labeled_returns_none() {
        let registry = dummy_registry();
        let resolved = SlotFilter::ShowLabeled.resolve(&registry, |_, _| {
            unreachable!("test bug: unknown token reached handler")
        });
        assert!(resolved.is_none());
    }

    /// `resolve_custom_maps_known_tokens_to_indices()` asserts that `Custom` resolves known
    /// tokens to their registry indices, preserving requested order
    ///
    #[test]
    fn resolve_custom_maps_known_tokens_to_indices() {
        let registry = dummy_registry();
        let filter = SlotFilter::Custom(vec!["GPU".to_string(), "OS".to_string()]);
        let resolved = filter
            .resolve(&registry, |_, _| {
                unreachable!("test bug: unknown token reached handler")
            })
            .unwrap();
        assert_eq!(resolved, vec![2, 0]);
    }

    /// `resolve_custom_is_case_insensitive()` asserts that `Custom` resolution is
    /// case-insensitive against registry tokens
    ///
    #[test]
    fn resolve_custom_is_case_insensitive() {
        let registry = dummy_registry();
        let filter = SlotFilter::Custom(vec!["cpu".to_string()]);
        let resolved = filter
            .resolve(&registry, |_, _| {
                unreachable!("test bug: unknown token reached handler")
            })
            .unwrap();
        assert_eq!(resolved, vec![1]);
    }

    /// `resolve_custom_unknown_token_invokes_handler()` asserts that an unknown token invokes
    /// the provided handler exactly once, with the offending token
    ///
    #[test]
    fn resolve_custom_unknown_token_invokes_handler() {
        let registry = dummy_registry();
        let filter = SlotFilter::Custom(vec!["BOGUS".to_string()]);
        let resolved = filter.resolve(&registry, |token, _registry| {
            assert_eq!(token, "BOGUS");
            usize::MAX
        });
        assert_eq!(resolved, Some(vec![usize::MAX]));
    }

    /// `resolve_custom_duplicate_tokens_are_currently_allowed()` asserts that duplicate tokens
    /// resolve to duplicate indices, matching the previous design's behavior
    ///
    #[test]
    fn resolve_custom_duplicate_tokens_are_currently_allowed() {
        let registry = dummy_registry();
        let filter = SlotFilter::Custom(vec!["OS".to_string(), "OS".to_string()]);
        let resolved = filter
            .resolve(&registry, |_, _| {
                unreachable!("test bug: unknown token reached handler")
            })
            .unwrap();
        assert_eq!(resolved, vec![0, 0]);
    }
}
