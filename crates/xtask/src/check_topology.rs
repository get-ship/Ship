//! `cargo xtask check-topology` — workspace topology invariants.
//!
//! Enforces the three checks ratified in `.artifacts/topology-proposal.md`:
//!
//! **(a) Nesting depth** — no workspace member may be nested 3+ crate-levels
//! deep (i.e. have 3+ `/crates/` segments in its workspace-relative path) except
//! `bin/` sub-crates, which are explicitly justified binary products.
//!
//! **(b) Single-consumer allowlist** — a workspace member with exactly ONE
//! intra-workspace non-dev consumer must appear in [`SINGLE_CONSUMER_ALLOWLIST`].
//! Any new single-consumer crate that is not allowlisted hard-fails CI, forcing
//! an explicit acknowledgment. The allowlist is seeded with the current legitimate
//! set discovered at topology-charter time (2026-06-10); it may only shrink as
//! pending moves (e.g. `ship-skill-vars`) complete.
//!
//! **(c) PASS summary** — prints member count + check summary on success.
//!
//! Run via `cargo xtask check-topology`. Exits 0 on clean, 1 on any violation.

use std::path::Path;

use crate::metadata;

// ── Single-consumer allowlist ─────────────────────────────────────────────────

/// Crates that currently have exactly ONE intra-workspace non-dev consumer and
/// are explicitly allowlisted for that topology.
///
/// Each entry has a rationale comment so future reviewers know why it was
/// admitted. The list MUST only shrink over time — never add a new entry
/// without a reviewer-signed justification.
///
/// Column: (crate-name, rationale summary)
const SINGLE_CONSUMER_ALLOWLIST: &[(&str, &str)] = &[
    // Axum HTTP transport adapter for the deck SSR pipeline.
    // Consumer: `deck` (the re-export facade). The facade is itself consumed by
    // many workspace members, so the effective consumer count is large; the
    // metadata resolver sees only the one direct non-dev edge to `deck`.
    ("deck-axum", "HTTP transport adapter for deck; consumer is the deck facade (many indirect consumers)"),
    // Durable-execution engine pending Wave-3 topology move (topology-proposal.md).
    // Currently nested under ship-substrate/crates/ but sole workspace consumer
    // is `floki` (L1). Wave-3 will promote it to crates/core/durable-execution/.
    ("durable-execution", "Wave-3 pending: will be promoted to crates/core peer; sole consumer floki is correct"),
    // Floki — the durable-execution runtime. Sole non-dev consumer today is
    // `shipd`; ship-pipeline-runtime dissolution (R13/#27) may add consumers.
    ("floki", "Durable-execution runtime; R13/#27 dissolution expected to add consumers"),
    // ship-messaging — DOCUMENTED OPERATOR EXCEPTION to Rule C2.
    // Intended extension-point crate; sole consumer today is `shipd` but
    // `ship-triggers` + Slack/WhatsApp adapters are expected consumers.
    // Cited in .artifacts/topology-proposal.md. Re-reviewed at next audit.
    ("ship-messaging", "Documented operator exception (topology-proposal.md): intended extension-point; expected consumers: ship-triggers, Slack, WhatsApp adapters"),
    // ship-search — dedicated search crate consumed only by `shipd`.
    // Kept separate so search indexing can be tested and extended independently.
    ("ship-search", "Search crate; single consumer shipd; deliberate separation for testability"),
    // ship-sidecar-proto — proto codegen crate (C1 forcing function: build.rs).
    // Sole consumer is `ship-isolation`.
    ("ship-sidecar-proto", "C1(b): build.rs proto codegen; sole consumer ship-isolation is correct"),
    // ship-skill-vars — PINNED pending Wave-1.1 move.
    // 198 LOC, sole consumer agent-def, NO C1 forcing function.
    // Should be dissolved into agent_def::skill_vars. Allowlist entry MAY ONLY
    // SHRINK — remove this entry once the Wave-1.1 move completes.
    ("ship-skill-vars", "Wave-1.1 PENDING: dissolve into agent_def::skill_vars module (3-level nesting violation; 198 LOC, no C1 forcing function)"),
    // sui-lint — linting rules for the sui-compiler, compiled as a separate
    // crate so linting passes can be loaded/tested independently.
    // Sole consumer: `sui-compiler`.
    ("sui-lint", "Independent linting pass crate for sui-compiler; sole consumer sui-compiler"),
];

// ── Depth check ───────────────────────────────────────────────────────────────

/// Count the number of `/crates/` directory segments in a workspace-relative
/// path. A path like `crates/core/ship-platform/crates/agent-def` has 2
/// segments and is at nesting depth 2 (one container per segment).
/// `crates/core/ship-platform/crates/agent-def/crates/ship-skill-vars` has 3 —
/// that is the 3-level violation Rule N1 prohibits.
fn crates_segment_depth(rel_dir: &str) -> usize {
    // Normalise separators.
    let p = rel_dir.replace('\\', "/");
    // Count "crates/" components — each one introduces a new layer of nesting.
    let mut depth = 0usize;
    let mut s = p.as_str();
    // The leading "crates/" at the workspace root is depth 1.
    if s.starts_with("crates/") {
        depth += 1;
        s = &s["crates/".len()..];
    }
    // Any subsequent "/crates/" segment is another nesting level.
    while let Some(pos) = s.find("/crates/") {
        depth += 1;
        s = &s[pos + "/crates/".len()..];
    }
    depth
}

/// Returns `true` if the path ends with a `bin/<name>` segment, marking it as
/// a justified binary sub-crate (Rule N1 exception).
fn is_bin_sub_crate(rel_dir: &str) -> bool {
    let p = rel_dir.replace('\\', "/");
    // Match `.../bin/<anything>` at the tail.
    if let Some(idx) = p.rfind("/bin/") {
        let after = &p[idx + "/bin/".len()..];
        // The remainder must be a single path component (no more slashes).
        return !after.contains('/') && !after.is_empty();
    }
    false
}

// ── Single-consumer check ─────────────────────────────────────────────────────

/// Build a map: crate-name → count of intra-workspace non-dev consumers.
fn consumer_counts(ws: &metadata::Workspace) -> std::collections::HashMap<String, usize> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for m in &ws.members {
        for dep in ws.direct_ws_dep_names(&m.name) {
            *counts.entry(dep).or_insert(0) += 1;
        }
    }
    counts
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(repo_root: &Path) -> Result<(), String> {
    let ws = metadata::load(repo_root)?;
    let counts = consumer_counts(&ws);

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ── (a) Nesting depth ────────────────────────────────────────────────────
    for m in &ws.members {
        let depth = crates_segment_depth(&m.rel_dir);
        if depth >= 3 && !is_bin_sub_crate(&m.rel_dir) {
            // The allowlisted ship-skill-vars is a KNOWN 3-level violation that
            // is pinned-pending-removal — emit a WARN rather than an error so
            // the allowlist entry can serve as a ratchet.
            let allowlisted = SINGLE_CONSUMER_ALLOWLIST
                .iter()
                .any(|(n, _)| *n == m.name.as_str());
            if allowlisted {
                warnings.push(format!(
                    "[N1-KNOWN] `{}` ({}) is nested {depth} crate-level(s) deep — known violation \
                     (allowlisted pending Wave-1.1 dissolution); remove when the move completes.",
                    m.name, m.rel_dir,
                ));
            } else {
                errors.push(format!(
                    "[N1] `{}` ({}) is nested {depth} crate-level(s) deep — Rule N1 allows at most \
                     2 levels (except bin/ sub-crates). Dissolve into a module or move up a layer.",
                    m.name, m.rel_dir,
                ));
            }
        }
    }

    // ── (b) Single-consumer allowlist ─────────────────────────────────────────
    let allowlist_names: std::collections::HashSet<&str> =
        SINGLE_CONSUMER_ALLOWLIST.iter().map(|(n, _)| *n).collect();

    // Crates with zero consumers are top-level apps/bins — not violations.
    // Crates with 2+ consumers need no entry.
    for m in &ws.members {
        let n = counts.get(&m.name).copied().unwrap_or(0);
        if n == 1 && !allowlist_names.contains(m.name.as_str()) {
            errors.push(format!(
                "[C2] `{}` ({}) has exactly ONE intra-workspace non-dev consumer but is not in the \
                 single-consumer allowlist. Either: (a) add it to SINGLE_CONSUMER_ALLOWLIST in \
                 crates/xtask/src/check_topology.rs with a justification, or (b) dissolve it into \
                 a module in its consumer (Rule C2).",
                m.name, m.rel_dir,
            ));
        }
    }

    // Warn about allowlist entries that are no longer single-consumer (the
    // allowlist should shrink, not grow stale entries).
    for (name, rationale) in SINGLE_CONSUMER_ALLOWLIST {
        let n = counts.get(*name).copied().unwrap_or(0);
        if ws.member(name).is_none() {
            // Crate was dissolved — the allowlist entry should be removed.
            warnings.push(format!(
                "[C2-STALE] allowlist entry `{name}` no longer exists in the workspace — \
                 remove it from SINGLE_CONSUMER_ALLOWLIST. Rationale was: {rationale}",
            ));
        } else if n != 1 {
            warnings.push(format!(
                "[C2-CHANGED] allowlist entry `{name}` now has {n} consumer(s) — \
                 the entry may be stale. Verify the rationale is still valid: {rationale}",
            ));
        }
    }

    // ── Report ───────────────────────────────────────────────────────────────
    for w in &warnings {
        eprintln!("WARN {w}");
    }
    if errors.is_empty() {
        let depth_ok = ws
            .members
            .iter()
            .filter(|m| crates_segment_depth(&m.rel_dir) < 3 || is_bin_sub_crate(&m.rel_dir))
            .count();
        println!(
            "check-topology PASS — {} workspace members; {} nesting-depth ok; \
             {} single-consumer allowlist entries; {} warnings",
            ws.members.len(),
            depth_ok,
            SINGLE_CONSUMER_ALLOWLIST.len(),
            warnings.len(),
        );
        return Ok(());
    }
    for e in &errors {
        eprintln!("  {e}");
    }
    Err(format!(
        "topology violations: {} (N1={}, C2={})",
        errors.len(),
        errors.iter().filter(|e| e.starts_with("[N1]")).count(),
        errors.iter().filter(|e| e.starts_with("[C2]")).count(),
    ))
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_counting_is_correct() {
        // Top-level app: no nesting at all.
        assert_eq!(crates_segment_depth("apps/shipd"), 0);
        // L0 root crate: one "crates/" prefix = depth 1.
        assert_eq!(crates_segment_depth("crates/core/ship-substrate"), 1);
        // L0 sub-crate: depth 2.
        assert_eq!(crates_segment_depth("crates/core/ship-substrate/crates/ship-types"), 2);
        // L1 sub-crate (under ship-platform): depth 2.
        assert_eq!(crates_segment_depth("crates/core/ship-platform/crates/ship-agents"), 2);
        // 3-level violation: depth 3.
        assert_eq!(
            crates_segment_depth("crates/core/ship-platform/crates/agent-def/crates/ship-skill-vars"),
            3
        );
    }

    #[test]
    fn bin_sub_crate_exception_matches() {
        assert!(is_bin_sub_crate("crates/core/ship-platform/crates/ship-isolation/bin/ship-sidecar"));
        assert!(!is_bin_sub_crate("crates/core/ship-substrate/crates/ship-types"));
        assert!(!is_bin_sub_crate("crates/core/ship-platform/crates/agent-def/crates/ship-skill-vars"));
    }

    #[test]
    fn depth_two_is_not_a_violation() {
        // Everything at 2 levels is fine (one container layer).
        assert!(crates_segment_depth("crates/core/ship-substrate/crates/ship-types") < 3);
    }
}
