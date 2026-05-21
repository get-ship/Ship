//! Deterministic `example()` fixtures for the daemon-side wire shapes — the
//! shipd counterpart to ship-platform's per-domain fixtures. `gen-fixtures`
//! merges these with `ship_platform::services::data_op_examples()`. Gated
//! behind `cfg(test)` / the `fixtures` feature.
//!
//! Covers every structurally-fixed `ship.*` data op assembled in the daemon.
//! The remaining `DataHandle<unknown>` ops (`collections.*`, `dashboard.state`,
//! `skills.summaries`) have `Value`-shaped returns; they still get a fixture
//! where a representative shape exists, but carry no struct-derived TS type.

mod git;
mod pipelines;
mod skills;
mod specs;
mod views;

use serde_json::Value;

/// Op name → example JSON for every daemon-assembled data op.
pub fn examples() -> Vec<(&'static str, Value)> {
    let mut out: Vec<(&'static str, Value)> = Vec::new();
    out.extend(views::examples());
    out.extend(skills::examples());
    out.extend(git::examples());
    out.extend(pipelines::examples());
    out.extend(specs::examples());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_examples_are_structured_and_unique() {
        let ex = examples();
        let mut seen = std::collections::HashSet::new();
        for (op, value) in &ex {
            assert!(seen.insert(*op), "duplicate fixture op: {op}");
            assert!(value.is_object() || value.is_array(), "{op} example is structured");
        }
        // chat/sessions/events/artifacts + workspaces(2) + settings + skills(4)
        // + git(6) + pipelines(6) + specs(4) = 27 daemon-side data ops.
        assert_eq!(ex.len(), 27, "expected 27 daemon-side fixtures, got {}", ex.len());
    }
}
