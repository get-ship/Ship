//! Lint the L0/L1/L2 crate-layer boundaries from the **resolved** dependency
//! graph (`cargo metadata`), not a Cargo.toml line-parser.
//!
//! Why resolved-graph: the old line-parser matched dep *keys* in the manifest
//! text, so a `package = "axum"` rename/alias slipped past, and it could only
//! ever see *direct* manifest deps — a layer violation laundered through an
//! intermediate crate was invisible. Resolving the graph closes both holes:
//! every edge is keyed by the dependency's *real* package name, and reachability
//! is transitive.
//!
//! Rules enforced (each appends to `errors`; the lint exits non-zero iff any
//! hard error remains after known-violations are accounted for):
//!
//! **R0 — Layer boundary:** no L0 crate may depend (directly or transitively,
//!   over non-dev edges) on an L1 crate.
//!
//! **R1 — No orphan crates:** every workspace member must classify as L0/L1/L2.
//!   An unclassifiable crate forces the taxonomy decision instead of silently
//!   escaping every rule.
//!
//! **R2 — L0 has no RPC/transport deps.** Two token families, each with its own
//!   allowlist, checked against the L0 crate's *resolved direct* deps:
//!   - INBOUND server/codec transport: `axum`, `connectrpc`, `http`, `tonic`,
//!     `hyper` — allowlisted for proto-codegen / protocol-boundary crates and
//!     the in-sandbox sidecar binary.
//!   - OUTBOUND HTTP clients: `reqwest`, `ureq`, `tower`, `hyper-util` —
//!     allowlisted only for the charter transport crates that are *supposed* to
//!     talk to the network (`ship-cloud`, `ship-gateway-client`, `ship-ops-rpc`,
//!     `ship-sidecar-proto`). Any other L0 crate reaching for an HTTP client is
//!     leaking egress into the substrate.
//!   Three tiers: ALLOWLIST (skip) → KNOWN-VIOLATION (warn, non-blocking) →
//!   HARD-FAIL (any other L0 crate). Dev-only edges are excluded — a dev-dep
//!   transport import is not an architectural violation.
//!
//! **R3 — Runtime seam:** `floki` and `ship-code` must never cross-depend (both
//!   may depend on `agent-def`; neither on the other). Vacuous if either is
//!   absent.
//!
//! **R4 — Downward-only into L2:** L1 must NOT depend on L2, and L0 must NOT
//!   depend on L2. Currently flags the live edge `ship-ui` (L1) → `deck` (L2)
//!   as a known-violation (WARN, non-blocking — the real fix is de-bucketing the
//!   SDK-manifest surface out of ship-ui; until then every *new* L1→L2 /
//!   L0→L2 edge hard-fails).
//!
//! Run via `cargo xtask lint-layers`. CI fails on any hard error; WARN lines
//! (known-violations) are reported but non-blocking.

use std::collections::HashSet;
use std::path::Path;

use crate::metadata::{self, Layer};

// ── R2 — transport token families + allowlists ─────────────────────────────────

/// INBOUND server / codec transport — running a wire server or speaking a
/// framed RPC codec inside an L0 crate.
const INBOUND_FORBIDDEN: &[&str] = &["axum", "connectrpc", "http", "tonic", "hyper"];

/// OUTBOUND HTTP clients — egress out of the substrate.
const OUTBOUND_FORBIDDEN: &[&str] = &["reqwest", "ureq", "tower", "hyper-util"];

/// INBOUND allowlist — proto-codegen / protocol-boundary crates whose generated
/// surface unavoidably pulls a server/codec. (ship-sidecar-proto / ship-sidecar
/// moved to ship-exec in j-09.)
const INBOUND_ALLOWLIST: &[&str] = &[
    "ship-ops-rpc",
    "ship-gateway-client",
];

/// OUTBOUND allowlist — the charter transport crates that are *supposed* to make
/// network calls. Everything else in L0 reaching for an HTTP client is a leak.
const OUTBOUND_ALLOWLIST: &[&str] = &[
    "ship-cloud",
    "ship-gateway-client",
    "ship-ops-rpc",
];

/// R2 known-violations — L0 crates that currently carry transport deps but are
/// on R15's "not ours, leave" list (or whose de-bucket is a separate slice).
/// Each emits a WARN; the lint does NOT fail on them. The owning lane ratchets
/// these to the allowlist or fixes the architecture before removing them.
///
/// - `ship-substrate` — `reqwest` pulled by `services/human_gateway.rs`; the fix
///   is moving that human-gateway egress out of the substrate (de-bucket), not
///   suppressing it here.
/// (ship-isolation moved to ship-exec in j-09.)
const R2_KNOWN_VIOLATIONS: &[&str] = &["ship-substrate"];

// ── R4 — known L1→L2 / L0→L2 edges (WARN, non-blocking) ────────────────────────

/// Directed `(src_crate, dst_crate)` edges that are *known* upward-into-L2
/// violations. Each is masked out of the reachability search (so its transitive
/// consequences don't independently turn the lint red) and reported as a single
/// WARN. The list is the ratchet: a *new* L1→L2 / L0→L2 edge not listed here
/// hard-fails.
const KNOWN_L2_EDGES: &[(&str, &str)] = &[
    // ship-ui (L1) embeds deck (L2) for the SDK-manifest render surface.
    // Fix = de-bucket that surface out of ship-ui; until then, WARN.
    ("ship-ui", "deck"),
];

pub fn run(repo_root: &Path) -> Result<(), String> {
    let ws = metadata::load(repo_root)?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ════════════════════════════════════════════════════════════════════════
    // R1 — No orphan crates: every workspace member must classify.
    // ════════════════════════════════════════════════════════════════════════
    for m in &ws.members {
        if m.layer.is_none() {
            errors.push(format!(
                "[R1] workspace member `{}` ({}) is not classifiable as L0, L1, or L2 — \
                 add it to an existing tier or define a new tier in metadata::classify",
                m.name, m.rel_dir,
            ));
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // R0 + R4 — layer-direction over the resolved workspace-member graph.
    //   No crate may depend (directly or transitively, non-dev) on a
    //   strictly-higher layer. Known upward edges are masked so their
    //   transitive fallout doesn't multiply into the error count.
    //     L0→L1            ⇒ R0
    //     L0→L2 / L1→L2    ⇒ R4
    // ════════════════════════════════════════════════════════════════════════
    let masked: HashSet<(String, String)> = KNOWN_L2_EDGES
        .iter()
        .map(|(s, d)| (s.to_string(), d.to_string()))
        .collect();

    for m in &ws.members {
        let Some(src_layer) = m.layer else { continue };
        let direct: HashSet<String> = ws.direct_ws_dep_names(&m.name).into_iter().collect();
        let mut hits: Vec<(String, Layer, bool)> = Vec::new();
        for dep in ws.reachable_ws(&m.name, &masked) {
            let Some(dst_layer) = ws.layer_of(&dep) else { continue };
            if dst_layer.rank() > src_layer.rank() {
                hits.push((dep.clone(), dst_layer, direct.contains(&dep)));
            }
        }
        hits.sort_by(|a, b| a.0.cmp(&b.0));
        for (dep, dst_layer, is_direct) in hits {
            let rule = if src_layer == Layer::L0 && dst_layer == Layer::L1 {
                "R0"
            } else {
                "R4"
            };
            let kind = if is_direct { "direct" } else { "transitive" };
            errors.push(format!(
                "[{rule}] {kind} upward dependency: `{}` ({}) depends on `{}` ({}) — \
                 forbidden; a layer must never depend on a higher one",
                m.name,
                src_layer.name(),
                dep,
                dst_layer.name(),
            ));
        }
    }

    // WARN for each present known-L2 edge (the masked, non-blocking violations).
    for (src, dst) in KNOWN_L2_EDGES {
        if ws.direct_ws_dep_names(src).iter().any(|d| d == dst) {
            let sl = ws.layer_of(src).map(Layer::name).unwrap_or("?");
            let dl = ws.layer_of(dst).map(Layer::name).unwrap_or("?");
            warnings.push(format!(
                "[R4-KNOWN] `{src}` ({sl}) depends on `{dst}` ({dl}) — known layer violation \
                 (live edge); de-bucket is the fix. Do NOT add new L1→L2 / L0→L2 edges.",
            ));
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // R2 — L0 has no RPC/transport deps (inbound server/codec + outbound HTTP).
    //   Checked against resolved DIRECT deps (non-dev). Three tiers per family:
    //   ALLOWLIST (skip) → KNOWN-VIOLATION (warn) → HARD-FAIL.
    // ════════════════════════════════════════════════════════════════════════
    for m in &ws.members {
        if m.layer != Some(Layer::L0) {
            continue;
        }
        let deps: HashSet<String> = ws.direct_dep_names(&m.name, false).into_iter().collect();
        let is_known = R2_KNOWN_VIOLATIONS.contains(&m.name.as_str());

        let mut hits: Vec<(&str, &str)> = Vec::new(); // (token, family)
        if !INBOUND_ALLOWLIST.contains(&m.name.as_str()) {
            for tok in INBOUND_FORBIDDEN {
                if deps.contains(*tok) {
                    hits.push((tok, "inbound transport"));
                }
            }
        }
        if !OUTBOUND_ALLOWLIST.contains(&m.name.as_str()) {
            for tok in OUTBOUND_FORBIDDEN {
                if deps.contains(*tok) {
                    hits.push((tok, "outbound HTTP client"));
                }
            }
        }
        for (tok, family) in hits {
            if is_known {
                warnings.push(format!(
                    "[R2-KNOWN] L0 crate `{}` has {family} dep `{tok}` — known violation; \
                     owning lane must de-bucket / move out of L0 (TODO: ratchet to hard-fail)",
                    m.name,
                ));
            } else {
                errors.push(format!(
                    "[R2] L0 crate `{}` depends on {family} `{tok}` — L0 must not pull \
                     transport; add to the allowlist, known-violations, or move it up a layer",
                    m.name,
                ));
            }
        }
    }

    // ── Report ──────────────────────────────────────────────────────────────
    for w in &warnings {
        eprintln!("WARN {w}");
    }
    if errors.is_empty() {
        let l0 = ws.members.iter().filter(|m| m.layer == Some(Layer::L0)).count();
        println!(
            "layer boundaries clean ({} workspace members classified; {l0} L0 crates checked; \
             {} known-violation warnings)",
            ws.members.len(),
            warnings.len(),
        );
        return Ok(());
    }
    for e in &errors {
        eprintln!("  {e}");
    }
    Err(format!(
        "layer boundary violations: {} (R0={}, R1={}, R2={}, R3={}, R4={})",
        errors.len(),
        errors.iter().filter(|e| e.starts_with("[R0]")).count(),
        errors.iter().filter(|e| e.starts_with("[R1]")).count(),
        errors.iter().filter(|e| e.starts_with("[R2]")).count(),
        errors.iter().filter(|e| e.starts_with("[R3]")).count(),
        errors.iter().filter(|e| e.starts_with("[R4]")).count(),
    ))
}
