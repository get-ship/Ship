//! Declarative diff-based gate — a structural primitive for pipeline steps.
//!
//! Pipelines may attach a `gate:` block with `require_changed` and
//! `forbid_changed` rules to a step. After the step's commits are sealed
//! the runtime computes the changed paths from `start_sha..HEAD` and
//! evaluates each rule:
//!
//! - `require_changed` — at least `min` files matching `match` (and not
//!   matching `not_match`) must be present.
//! - `forbid_changed` — any matching file fails the gate.
//!
//! The evaluator is data-driven and pure — same inputs, same outcome,
//! no fs / network / clock. Suitable for the cloud worker and the
//! eventual browser embed.

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

/// Declarative diff-based gate attached to a step. Runtime evaluates
/// after the step's commits are sealed, before the next step is dispatched.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GateSpec {
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_rules")]
    pub require_changed: Vec<GateRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_rules")]
    pub forbid_changed: Vec<GateRule>,
}

/// A single gate rule. `match` and `not_match` are lists of glob patterns.
///
/// Canonical YAML/JSON shape (e.g. match all `.rs` files, exclude tests,
/// require at least one change):
///
///   match:     two-star slash star-dot-rs and star-dot-toml
///   not_match: two-star slash tests slash two-star
///   min:       1
///
/// Earlier revisions accepted a pipe-separated string for match/not_match
/// and a single rule object for require_changed/forbid_changed; both
/// shorthand forms are rejected at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GateRule {
    #[serde(rename = "match", deserialize_with = "deserialize_patterns")]
    pub match_patterns: Vec<String>,
    #[serde(default, rename = "not_match",
            deserialize_with = "deserialize_patterns")]
    pub not_match: Vec<String>,
    #[serde(default = "default_min")]
    pub min: u32,
}

fn default_min() -> u32 {
    1
}

/// Canonical: a list of rule objects. The legacy single-object shorthand is
/// rejected with an error pointing at the canonical syntax.
fn deserialize_rules<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<GateRule>, D::Error> {
    use serde::de::Error;
    let val = serde_json::Value::deserialize(d)?;
    if !val.is_array() {
        return Err(D::Error::custom(
            "gate rule list must be a YAML/JSON array of rule objects \
             (e.g. `require_changed: [{ match: [\"**/*.rs\"], min: 1 }]`); \
             the legacy single-object shorthand was removed in the v1 \
             contract freeze",
        ));
    }
    serde_json::from_value(val).map_err(D::Error::custom)
}

/// Canonical: a list of glob strings. The legacy pipe-separated string
/// shorthand is rejected with an error pointing at the canonical syntax.
fn deserialize_patterns<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    use serde::de::Error;
    let val = serde_json::Value::deserialize(d)?;
    match val {
        serde_json::Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                match item {
                    serde_json::Value::String(s) => {
                        if s.contains('|') {
                            return Err(D::Error::custom(format!(
                                "gate pattern[{idx}] '{s}' contains '|'; \
                                 pipe-separated patterns were removed in the \
                                 v1 contract freeze. Split into separate list \
                                 entries: `match: [\"a\", \"b\"]`"
                            )));
                        }
                        if s.is_empty() {
                            return Err(D::Error::custom(format!(
                                "gate pattern[{idx}] is empty"
                            )));
                        }
                        out.push(s);
                    }
                    other => return Err(D::Error::custom(format!(
                        "gate pattern[{idx}] must be a glob string, got {other:?}"
                    ))),
                }
            }
            Ok(out)
        }
        serde_json::Value::String(_) => Err(D::Error::custom(
            "gate `match`/`not_match` must be a list of glob strings \
             (e.g. `match: [\"**/*.rs\", \"**/*.toml\"]`); the legacy \
             pipe-separated string shorthand was removed in the v1 \
             contract freeze",
        )),
        other => Err(D::Error::custom(format!(
            "gate `match`/`not_match` must be a list of glob strings, got {other:?}"
        ))),
    }
}

/// Result of gate evaluation.
pub enum GateOutcome {
    /// No gate declared, or all rules satisfied.
    Pass,
    /// Gate failed — includes a structured error message for pipeline halt.
    Fail(String),
}

/// Evaluate a gate against a set of changed paths. The commit range (as
/// `start_sha..HEAD`) is included in failure messages for operator debuggability.
pub fn evaluate(
    gate: Option<&GateSpec>,
    changed: &[String],
    commit_range: &str,
) -> GateOutcome {
    let Some(g) = gate else { return GateOutcome::Pass };
    if g.require_changed.is_empty() && g.forbid_changed.is_empty() {
        return GateOutcome::Pass;
    }

    let require = match compile_rules(&g.require_changed) {
        Ok(r) => r,
        Err(e) => return GateOutcome::Fail(format!("gate: invalid require_changed pattern: {e}")),
    };
    let forbid = match compile_rules(&g.forbid_changed) {
        Ok(r) => r,
        Err(e) => return GateOutcome::Fail(format!("gate: invalid forbid_changed pattern: {e}")),
    };

    for (idx, (rule, compiled)) in g.require_changed.iter().zip(require.iter()).enumerate() {
        let matched: Vec<&String> = changed.iter()
            .filter(|p| compiled.match_set.is_match(p.as_str()))
            .filter(|p| !compiled.not_set.is_match(p.as_str()))
            .collect();
        if (matched.len() as u32) < rule.min {
            return GateOutcome::Fail(format_require_failure(
                idx, rule, &matched, changed, commit_range,
            ));
        }
    }

    for (idx, (rule, compiled)) in g.forbid_changed.iter().zip(forbid.iter()).enumerate() {
        let hit: Vec<&String> = changed.iter()
            .filter(|p| compiled.match_set.is_match(p.as_str()))
            .filter(|p| !compiled.not_set.is_match(p.as_str()))
            .collect();
        if !hit.is_empty() {
            return GateOutcome::Fail(format_forbid_failure(
                idx, rule, &hit, changed, commit_range,
            ));
        }
    }

    GateOutcome::Pass
}

/// Validate a gate at load time. Fails fast on unresolvable globs / negative
/// `min` so pipeline authors see errors before dispatch.
pub fn validate(gate: &GateSpec) -> Result<(), String> {
    for r in gate.require_changed.iter().chain(gate.forbid_changed.iter()) {
        if r.match_patterns.is_empty() {
            return Err("gate rule has no `match` pattern".to_string());
        }
        for p in r.match_patterns.iter().chain(r.not_match.iter()) {
            Glob::new(p).map_err(|e| format!("gate: invalid glob `{p}`: {e}"))?;
        }
    }
    Ok(())
}

struct Compiled {
    match_set: GlobSet,
    not_set: GlobSet,
}

fn compile_rules(rules: &[GateRule]) -> Result<Vec<Compiled>, String> {
    let mut out = Vec::with_capacity(rules.len());
    for r in rules {
        let mut mb = GlobSetBuilder::new();
        for p in &r.match_patterns {
            mb.add(Glob::new(p).map_err(|e| format!("`{p}`: {e}"))?);
        }
        let match_set = mb.build().map_err(|e| format!("build match: {e}"))?;
        let mut nb = GlobSetBuilder::new();
        for p in &r.not_match {
            nb.add(Glob::new(p).map_err(|e| format!("`{p}`: {e}"))?);
        }
        let not_set = nb.build().map_err(|e| format!("build not: {e}"))?;
        out.push(Compiled { match_set, not_set });
    }
    Ok(out)
}

fn format_require_failure(
    idx: usize,
    rule: &GateRule,
    matched: &[&String],
    all_changed: &[String],
    commit_range: &str,
) -> String {
    format!(
        "gate require_changed[{idx}] failed: {} of {} required matches in {} \
         (rule: match={:?} not_match={:?} min={}; matched: {:?}; all_changed: {:?})",
        matched.len(),
        rule.min,
        commit_range,
        rule.match_patterns,
        rule.not_match,
        rule.min,
        matched.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        all_changed,
    )
}

fn format_forbid_failure(
    idx: usize,
    rule: &GateRule,
    hit: &[&String],
    all_changed: &[String],
    commit_range: &str,
) -> String {
    format!(
        "gate forbid_changed[{idx}] failed: {} forbidden match(es) in {} \
         (rule: match={:?} not_match={:?}; hit: {:?}; all_changed: {:?})",
        hit.len(),
        commit_range,
        rule.match_patterns,
        rule.not_match,
        hit.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        all_changed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(match_pats: &[&str], not_match: &[&str], min: u32) -> GateRule {
        GateRule {
            match_patterns: match_pats.iter().map(|s| s.to_string()).collect(),
            not_match: not_match.iter().map(|s| s.to_string()).collect(),
            min,
        }
    }

    #[test]
    fn no_gate_passes() {
        assert!(matches!(evaluate(None, &[], "abc..def"), GateOutcome::Pass));
    }

    #[test]
    fn empty_spec_passes() {
        let g = GateSpec::default();
        assert!(matches!(
            evaluate(Some(&g), &["x.rs".into()], "abc..def"),
            GateOutcome::Pass
        ));
    }

    #[test]
    fn require_changed_min_satisfied_passes() {
        let g = GateSpec {
            require_changed: vec![rule(&["**/*.rs"], &[], 1)],
            forbid_changed: vec![],
        };
        let changed = vec!["src/lib.rs".to_string(), "README.md".to_string()];
        assert!(matches!(evaluate(Some(&g), &changed, "abc..def"), GateOutcome::Pass));
    }

    #[test]
    fn require_changed_min_not_met_fails() {
        let g = GateSpec {
            require_changed: vec![rule(&["**/*.rs"], &[], 2)],
            forbid_changed: vec![],
        };
        let changed = vec!["src/lib.rs".to_string()];
        match evaluate(Some(&g), &changed, "abc..def") {
            GateOutcome::Fail(msg) => assert!(msg.contains("require_changed[0] failed")),
            _ => panic!("expected fail"),
        }
    }

    #[test]
    fn forbid_changed_hit_fails() {
        let g = GateSpec {
            require_changed: vec![],
            forbid_changed: vec![rule(&["secret/**"], &[], 1)],
        };
        let changed = vec!["secret/key.txt".to_string()];
        match evaluate(Some(&g), &changed, "abc..def") {
            GateOutcome::Fail(msg) => assert!(msg.contains("forbid_changed[0] failed")),
            _ => panic!("expected fail"),
        }
    }

    #[test]
    fn validate_rejects_empty_match_pattern() {
        let g = GateSpec {
            require_changed: vec![rule(&[], &[], 1)],
            forbid_changed: vec![],
        };
        assert!(validate(&g).is_err());
    }

    #[test]
    fn validate_rejects_invalid_glob() {
        let g = GateSpec {
            require_changed: vec![rule(&["[bad-glob"], &[], 1)],
            forbid_changed: vec![],
        };
        assert!(validate(&g).is_err());
    }

    fn paths(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    fn is_fail(o: &GateOutcome) -> bool {
        matches!(o, GateOutcome::Fail(_))
    }

    fn reason(o: &GateOutcome) -> String {
        match o {
            GateOutcome::Fail(s) => s.clone(),
            _ => String::new(),
        }
    }

    #[test]
    fn require_changed_satisfied_with_mixed_diff() {
        let g = GateSpec {
            require_changed: vec![rule(&["**/*.rs"], &["**/tests/**"], 1)],
            forbid_changed: vec![],
        };
        let out = evaluate(
            Some(&g),
            &paths(&["crates/x/src/lib.rs", "crates/x/tests/t.rs", "README.md"]),
            "a..b",
        );
        assert!(matches!(out, GateOutcome::Pass));
    }

    #[test]
    fn require_changed_fails_when_only_tests_match() {
        let g = GateSpec {
            require_changed: vec![rule(&["**/*.rs"], &["**/tests/**"], 1)],
            forbid_changed: vec![],
        };
        let out = evaluate(
            Some(&g),
            &paths(&["crates/x/tests/t.rs", "README.md"]),
            "abc..def",
        );
        assert!(is_fail(&out));
        let r = reason(&out);
        assert!(r.contains("require_changed[0]"), "{r}");
        assert!(r.contains("abc..def"), "{r}");
    }

    #[test]
    fn multiple_require_rules_and_together_fail_on_any() {
        let g = GateSpec {
            require_changed: vec![
                rule(&["**/*.rs"], &[], 1),
                rule(&["crates/core/runtime/**"], &[], 1),
            ],
            forbid_changed: vec![],
        };
        let out = evaluate(
            Some(&g),
            &paths(&["crates/other/src/lib.rs"]),
            "a..b",
        );
        assert!(is_fail(&out));
        assert!(reason(&out).contains("require_changed[1]"));
    }

    #[test]
    fn not_match_excludes_test_paths() {
        let g = GateSpec {
            require_changed: vec![rule(
                &["**/*.rs", "**/*.ts", "**/*.tsx", "**/*.py"],
                &["**/tests/**", "**/*.test.*", "**/*.spec.*"],
                1,
            )],
            forbid_changed: vec![],
        };
        let tests_only = paths(&["crates/x/tests/t.rs", "src/foo.test.ts"]);
        assert!(is_fail(&evaluate(Some(&g), &tests_only, "a..b")));

        let mixed = paths(&["crates/x/src/lib.rs", "src/foo.test.ts"]);
        assert!(matches!(evaluate(Some(&g), &mixed, "a..b"), GateOutcome::Pass));
    }
}
