//! Post-step diff gate — generic structural primitive.
//!
//! A pipeline step may declare a `gate:` block with `require_changed` and
//! `forbid_changed` rules. After the step's commits are sealed, the runtime
//! computes the changed paths from `start_sha..HEAD` and evaluates each rule:
//!
//! - `require_changed` — at least `min` files matching `match` (and not
//!   matching `not_match`) must be present.
//! - `forbid_changed` — any matching file fails the gate.
//!
//! The gate is data. Pipelines declare it in YAML; runtime evaluates without
//! any task-specific knowledge.

use crate::events::job::{GateRule, GateSpec};
use globset::{Glob, GlobSet, GlobSetBuilder};

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

    // Precompile every rule. Malformed globs are a hard failure — the rule
    // is unusable and the pipeline should not advance.
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
    rules.iter().map(compile_rule).collect()
}

fn compile_rule(rule: &GateRule) -> Result<Compiled, String> {
    let match_set = build_set(&rule.match_patterns)?;
    let not_set = build_set(&rule.not_match)?;
    Ok(Compiled { match_set, not_set })
}

fn build_set(patterns: &[String]) -> Result<GlobSet, String> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        let g = Glob::new(p).map_err(|e| format!("invalid glob `{p}`: {e}"))?;
        b.add(g);
    }
    b.build().map_err(|e| e.to_string())
}

fn format_require_failure(
    idx: usize, rule: &GateRule, matched: &[&String],
    all: &[String], commit_range: &str,
) -> String {
    format!(
        "gate: require_changed[{idx}] not satisfied ({matched_count}/{min} in {range}): \
         match={match_pats:?} not_match={not_pats:?}; \
         matched=[{matched}]; all_changed=[{all}]",
        matched_count = matched.len(),
        min = rule.min,
        range = commit_range,
        match_pats = rule.match_patterns,
        not_pats = rule.not_match,
        matched = join(matched),
        all = all.join(","),
    )
}

fn format_forbid_failure(
    idx: usize, rule: &GateRule, hit: &[&String],
    all: &[String], commit_range: &str,
) -> String {
    format!(
        "gate: forbid_changed[{idx}] triggered in {range}: \
         match={match_pats:?} not_match={not_pats:?}; \
         forbidden=[{hit}]; all_changed=[{all}]",
        range = commit_range,
        match_pats = rule.match_patterns,
        not_pats = rule.not_match,
        hit = join(hit),
        all = all.join(","),
    )
}

fn join(v: &[&String]) -> String {
    v.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
}

/// Compute changed file paths between two SHAs (inclusive of everything the
/// step committed). Returns empty if git fails — the caller's `require_changed`
/// rules will then naturally fail with an empty path list.
pub fn diff_paths(work_dir: &std::path::Path, start_sha: &str) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{start_sha}..HEAD")])
        .current_dir(work_dir)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines().filter(|l| !l.is_empty()).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(m: &[&str], not: &[&str], min: u32) -> GateRule {
        GateRule {
            match_patterns: m.iter().map(|s| s.to_string()).collect(),
            not_match: not.iter().map(|s| s.to_string()).collect(),
            min,
        }
    }

    fn paths(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    fn is_fail(o: &GateOutcome) -> bool { matches!(o, GateOutcome::Fail(_)) }
    fn reason(o: &GateOutcome) -> String {
        match o { GateOutcome::Fail(s) => s.clone(), _ => String::new() }
    }

    #[test]
    fn no_gate_is_noop() {
        assert!(matches!(evaluate(None, &paths(&["x.rs"]), "a..b"), GateOutcome::Pass));
    }

    #[test]
    fn empty_gate_is_noop() {
        let g = GateSpec::default();
        assert!(matches!(evaluate(Some(&g), &paths(&["x.rs"]), "a..b"), GateOutcome::Pass));
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
        assert!(r.contains("tests/t.rs"), "{r}");
        assert!(r.contains("README.md"), "{r}");
    }

    #[test]
    fn forbid_changed_triggers_on_match() {
        let g = GateSpec {
            require_changed: vec![],
            forbid_changed: vec![rule(&["**/*.lock"], &[], 1)],
        };
        let out = evaluate(
            Some(&g),
            &paths(&["Cargo.lock", "src/x.rs"]),
            "a..b",
        );
        assert!(is_fail(&out));
        assert!(reason(&out).contains("forbid_changed[0]"));
        assert!(reason(&out).contains("Cargo.lock"));
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
        // First rule satisfied, second not.
        let out = evaluate(
            Some(&g),
            &paths(&["crates/other/src/lib.rs"]),
            "a..b",
        );
        assert!(is_fail(&out));
        assert!(reason(&out).contains("require_changed[1]"));
    }

    #[test]
    fn tdd_style_pipe_separated_not_match_excludes_tests() {
        // Simulates parsing of `not_match: "**/tests/**|**/*.test.*|**/*.spec.*"`.
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

    #[test]
    fn validate_catches_empty_match() {
        let g = GateSpec {
            require_changed: vec![GateRule { match_patterns: vec![], not_match: vec![], min: 1 }],
            forbid_changed: vec![],
        };
        assert!(validate(&g).is_err());
    }
}
