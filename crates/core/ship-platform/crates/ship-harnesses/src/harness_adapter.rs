//! Harness adapter registry — maps harness names to their adapters.
//!
//! The trait/types live in `ship_runner::harness_adapter` (L0). This module
//! owns the REGISTRY slice that references the concrete adapter structs and
//! exposes the resolver helpers (`adapter_for`, `capabilities`). It re-exports
//! all L0 types so existing `ship_harnesses::harness_adapter::*` consumers
//! continue to compile without change.

pub use ship_runner::harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError,
};

/// The single source of truth for which harnesses exist. `adapter_for`, the
/// `Known:` error string, and `capabilities()` all derive from this slice —
/// adding or dropping a harness is one edit here plus the adapter module, not
/// a sweep across parallel match arms and enumerations. Adapters are zero-
/// sized unit structs, so this is a `&'static` slice of const-promoted refs
/// with no allocation.
const REGISTRY: &[&dyn HarnessAdapter] = &[
    &crate::harnesses::claude::ClaudeAdapter,
    &crate::harnesses::codex::CodexAdapter,
    &crate::harnesses::opencode::OpenCodeAdapter,
    &crate::harnesses::pi::PiAdapter,
    &crate::harnesses::omp::OmpAdapter,
];

/// Comma-separated list of canonical harness ids, for error messages.
fn known_ids() -> String {
    REGISTRY
        .iter()
        .map(|a| a.id())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolve a harness name to its adapter.
/// Returns Err for unknown harnesses (no silent default) so a `foo-typo`
/// config doesn't silently run as Claude with weird flags.
pub fn adapter_for(harness: &str) -> Result<&'static dyn HarnessAdapter, String> {
    REGISTRY
        .iter()
        .copied()
        .find(|a| a.matches(harness))
        .ok_or_else(|| format!("unknown harness '{harness}'. Known: {}.", known_ids()))
}

/// Convenience: resolve an Option<&str>, falling back to "claude" when None.
/// Used by executors that want a default for unconfigured pipelines.
pub fn adapter_for_or_default(
    harness: Option<&str>,
) -> Result<&'static dyn HarnessAdapter, String> {
    adapter_for(harness.unwrap_or("claude"))
}

/// Enumerate all known harnesses and their capabilities.
/// Used by `ship status --harnesses` and Studio's harness picker.
pub fn capabilities() -> Vec<(&'static str, HarnessCapabilities)> {
    REGISTRY.iter().map(|a| (a.id(), a.caps())).collect()
}

#[cfg(test)]
mod adapter_for_tests {
    use super::*;

    #[test]
    fn all_harnesses_advertise_persistent_session() {
        for name in ["claude", "codex", "opencode", "pi", "omp"] {
            let adapter = adapter_for(name).unwrap();
            assert!(
                adapter.caps().persistent_session,
                "harness '{name}' must support persistent_session"
            );
        }
    }

    #[test]
    fn push_mechanism_per_harness() {
        use PushMechanism::*;
        let expected: &[(&str, PushMechanism)] = &[
            ("claude", Channel),
            ("codex", Stdin),
            ("opencode", Stdin),
            ("pi", Stdin),
            ("omp", Stdin),
        ];
        for (name, want) in expected {
            assert_eq!(
                adapter_for(name).unwrap().caps().push_mechanism,
                *want,
                "harness '{name}' push_mechanism drifted"
            );
        }
    }

    #[test]
    fn adapter_for_codex_returns_codex_adapter() {
        assert_eq!(adapter_for("codex").unwrap().id(), "codex");
    }

    #[test]
    fn adapter_for_opencode_returns_opencode_adapter() {
        assert_eq!(adapter_for("opencode").unwrap().id(), "opencode");
    }

    #[test]
    fn adapter_for_omp_returns_omp_adapter() {
        // omp is the pi fork; it must resolve to its own adapter (binary `omp`),
        // not silently fall through to pi.
        let adapter = adapter_for("omp").unwrap();
        assert_eq!(adapter.id(), "omp");
        assert_eq!(adapter.binary(), "omp");
    }

    #[test]
    fn adapter_for_or_default_none_returns_claude_adapter() {
        assert_eq!(adapter_for_or_default(None).unwrap().id(), "claude");
    }

    #[test]
    fn adapter_for_unknown_returns_err() {
        let result = adapter_for("nonexistent");
        assert!(result.is_err());
        let msg = result.err().unwrap();
        assert!(msg.contains("unknown harness"), "msg was: {msg}");
        assert!(msg.contains("Known: claude, codex"), "msg was: {msg}");
    }

    #[test]
    fn adapter_for_or_default_propagates_unknown_err() {
        let result = adapter_for_or_default(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn binary_defaults_to_id() {
        for (id, _) in capabilities() {
            assert_eq!(adapter_for(id).unwrap().binary(), id, "binary() wrong for {id}");
        }
    }

    #[test]
    fn registry_is_single_source_of_truth() {
        let known = adapter_for("definitely-not-a-harness")
            .err()
            .expect("unknown harness must error");
        for (id, _caps) in capabilities() {
            assert_eq!(adapter_for(id).unwrap().id(), id, "adapter_for disagrees for {id}");
            assert!(known.contains(id), "Known: list missing {id}: {known}");
        }
    }

    #[test]
    fn capabilities_lists_registered_harnesses() {
        let caps = capabilities();
        let ids: Vec<&str> = caps.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec!["claude", "codex", "opencode", "pi", "omp"]);
    }

    #[test]
    fn codex_declares_server_mode() {
        let caps = capabilities();
        let server_capable: Vec<&str> = caps
            .iter()
            .filter(|(_, c)| c.server_mode)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(server_capable, vec!["codex"]);
    }

    #[test]
    fn claude_does_not_declare_server_mode() {
        let claude = adapter_for("claude").unwrap();
        assert!(!claude.caps().server_mode);
    }

    #[test]
    fn minimal_caps_are_conservative() {
        let m = HarnessCapabilities::MINIMAL;
        assert!(m.headless);
        assert!(m.interactive);
        assert!(!m.server_mode);
        assert!(!m.json_stream);
        assert!(!m.mcp);
        assert!(!m.hooks);
        assert!(!m.session_resume);
        assert!(!m.persistent_session);
        assert_eq!(m.push_mechanism, PushMechanism::Respawn);
    }

    #[test]
    fn only_claude_uses_channel_push() {
        let caps = capabilities();
        let channel_users: Vec<&str> = caps
            .iter()
            .filter(|(_, c)| c.push_mechanism == PushMechanism::Channel)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(channel_users, vec!["claude"]);
    }
}
