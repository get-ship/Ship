//! Shared `cargo metadata` loader for the lint subcommands.
//!
//! Every layer / import / residue lint derives its crate + path lists from this
//! module instead of hardcoded path strings. The old failure mode this kills: a
//! skip-if-dir-missing guard that let a *rotted* configured path (a crate that
//! had been renamed or relocated) silently drop out of a rule's coverage — the
//! rule kept "passing" while actually checking nothing. Deriving from the
//! resolved workspace means a moved crate auto-follows, and an unknown crate
//! name screams (the consuming lint turns it into a hard error).
//!
//! Layer classification is path-based (workspace-relative crate directory) so
//! it never drifts from the filesystem, and is computed once here so every
//! lint agrees on the taxonomy.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// Coarse architecture layer used by the layer-boundary lints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// L0 — substrate tree + sibling primitives (no upward deps allowed).
    L0,
    /// L1 — platform primitives (may depend on L0; not on L2).
    L1,
    /// L2 — application / tooling layer (top of the stack).
    L2,
}

impl Layer {
    pub fn rank(self) -> u8 {
        match self {
            Layer::L0 => 0,
            Layer::L1 => 1,
            Layer::L2 => 2,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Layer::L0 => "L0",
            Layer::L1 => "L1",
            Layer::L2 => "L2",
        }
    }
}

/// L0 sibling primitives that live under `crates/core/` next to ship-substrate.
const L0_SIBLINGS: &[&str] = &[
    "ship-git",
    "ship-isolation",
    "ship-cloud",
    "ship-ops-rpc",
    "ship-gateway-client",
    "ship-sidecar-proto",
    "stack-model",
    "agent-config-eval",
];

/// Classify a workspace-relative crate directory into a layer.
///
/// Returns `None` for an unclassifiable member — the caller surfaces that as an
/// R1 orphan error (the taxonomy decision is forced rather than guessed).
pub fn classify(rel_dir: &str) -> Option<Layer> {
    // The code world (crates/code/) — ship-code's tier assignments carry
    // over from their pre-move homes: harness-config-compiler was an L0 sibling
    // (pure logic), ship-code an L1 platform crate. The standard/code
    // boundary itself is lint-boundary's (path-based, transitive); tiers
    // here only feed the R2 ordering checks.
    if rel_dir == "crates/code/harness-config-compiler"
        || rel_dir.starts_with("crates/code/harness-config-compiler/")
    {
        return Some(Layer::L0);
    }
    if rel_dir == "crates/code/ship-harnesses"
        || rel_dir.starts_with("crates/code/ship-harnesses/")
    {
        // Concrete third-party-harness driver impls (L0 sibling — peer to
        // `agent-driver`'s contract; depends on ship-substrate, not on
        // ship-platform).
        return Some(Layer::L0);
    }
    // L0 — ship-substrate tree (root + every nested crate).
    if rel_dir == "crates/core/ship-substrate" || rel_dir.starts_with("crates/core/ship-substrate/")
    {
        return Some(Layer::L0);
    }
    // L0 — sibling primitives under crates/core/.
    for sib in L0_SIBLINGS {
        let prefix = format!("crates/core/{sib}");
        if rel_dir == prefix || rel_dir.starts_with(&format!("{prefix}/")) {
            return Some(Layer::L0);
        }
    }
    // L1 — ship-platform tree + the pipeline runtime.
    if rel_dir == "crates/core/ship-platform"
        || rel_dir.starts_with("crates/core/ship-platform/")
        || rel_dir == "crates/ship-pipeline-runtime"
    {
        return Some(Layer::L1);
    }
    // L1 — ship-resolver (Phase-2 space-shape resolver; orchestrates
    // substrate ports + stack-model). Carved from ship-platform but lives
    // at the L0 sibling root for cycle reasons (depends on
    // ship-store/ship-substrate/compiler, NOT on ship-platform), so it
    // gets its own explicit entry.
    if rel_dir == "crates/core/ship-resolver" {
        return Some(Layer::L1);
    }
    // L2 — apps, packages, tests, examples, plus xtask + build-info tooling.
    // crates/deck/* + crates/sui/* moved to the external sui repo (post-carve);
    // the xtask no longer classifies them in the ship layer graph.
    if rel_dir.starts_with("apps/")
        || rel_dir.starts_with("packages/")
        || rel_dir.starts_with("tests/")
        || rel_dir.starts_with("examples/")
        || rel_dir == "crates/xtask"
        || rel_dir == "crates/build-info"
    {
        return Some(Layer::L2);
    }
     None
 }
// ── Raw `cargo metadata --format-version 1` shapes (only fields we read). ──────

#[derive(Deserialize)]
struct RawMetadata {
    packages: Vec<RawPackage>,
    workspace_members: Vec<String>,
    workspace_root: String,
    resolve: RawResolve,
}

#[derive(Deserialize)]
struct RawPackage {
    id: String,
    name: String,
    manifest_path: String,
    targets: Vec<RawTarget>,
}

#[derive(Deserialize)]
struct RawTarget {
    src_path: String,
}

#[derive(Deserialize)]
struct RawResolve {
    nodes: Vec<RawNode>,
}

#[derive(Deserialize)]
struct RawNode {
    id: String,
    deps: Vec<RawDep>,
}

#[derive(Deserialize)]
struct RawDep {
    pkg: String,
    dep_kinds: Vec<RawDepKind>,
}

#[derive(Deserialize)]
struct RawDepKind {
    /// `null` = normal, `"dev"` = dev-dependency, `"build"` = build-dependency.
    kind: Option<String>,
}

// ── Public model ──────────────────────────────────────────────────────────────

/// One workspace member crate.
pub struct Member {
    pub name: String,
    /// Absolute crate directory (the dir containing Cargo.toml).
    pub dir: PathBuf,
    /// Workspace-relative crate directory (e.g. `crates/core/ship-substrate`).
    pub rel_dir: String,
    /// `dir/src` (may not exist for proc-macro-free metadata-only crates).
    pub src_dir: PathBuf,
    /// Layer classification (`None` ⇒ R1 orphan).
    pub layer: Option<Layer>,
    /// Absolute `src_path`s of this crate's compiler targets (lib/bin/test/…).
    pub target_src_paths: Vec<PathBuf>,
}

/// Resolved workspace: members + the resolved dependency graph. Built once and
/// shared by every lint so they agree on crate identity and dependency edges.
pub struct Workspace {
    pub root: PathBuf,
    pub members: Vec<Member>,
    /// package id → crate name (covers ALL packages, incl. transitive externals;
    /// resolving a dep's id to its *real* name closes package-rename/alias evasion).
    id_to_name: HashMap<String, String>,
    /// workspace-member package ids.
    ws_ids: HashSet<String>,
    /// crate name → package id (workspace members only).
    name_to_id: HashMap<String, String>,
    /// package id → its resolved direct deps as `(dep pkg id, dev_only)`.
    node_deps: HashMap<String, Vec<(String, bool)>>,
}

impl Workspace {
    /// Look up a member by crate name.
    pub fn member(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    /// Layer of a workspace member by name (`None` if absent or unclassifiable).
    pub fn layer_of(&self, name: &str) -> Option<Layer> {
        self.member(name).and_then(|m| m.layer)
    }

    /// Resolved direct dependency crate names of `name`. When `include_dev` is
    /// false, dev-only deps are excluded. Names are the deps' *real* package
    /// names (resolved through the id), so a `package = "…"` rename cannot hide.
    pub fn direct_dep_names(&self, name: &str, include_dev: bool) -> Vec<String> {
        let Some(id) = self.name_to_id.get(name) else {
            return Vec::new();
        };
        let Some(deps) = self.node_deps.get(id) else {
            return Vec::new();
        };
        let mut out: Vec<String> = deps
            .iter()
            .filter(|(_, dev_only)| include_dev || !dev_only)
            .filter_map(|(dep_id, _)| self.id_to_name.get(dep_id).cloned())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Resolved direct dependency *workspace-member* crate names of `name`
    /// (non-dev edges only).
    pub fn direct_ws_dep_names(&self, name: &str) -> Vec<String> {
        let Some(id) = self.name_to_id.get(name) else {
            return Vec::new();
        };
        let Some(deps) = self.node_deps.get(id) else {
            return Vec::new();
        };
        let mut out: Vec<String> = deps
            .iter()
            .filter(|(_, dev_only)| !dev_only)
            .filter(|(dep_id, _)| self.ws_ids.contains(dep_id))
            .filter_map(|(dep_id, _)| self.id_to_name.get(dep_id).cloned())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// All workspace members transitively reachable from `name` over non-dev
    /// edges, skipping any `(src, dst)` edge in `masked`. Masking the known
    /// upward edges lets a single allowlisted violation absorb its transitive
    /// consequences without those consequences each turning the lint red.
    pub fn reachable_ws(&self, name: &str, masked: &HashSet<(String, String)>) -> HashSet<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = vec![name.to_string()];
        while let Some(cur) = stack.pop() {
            for dep in self.direct_ws_dep_names(&cur) {
                if masked.contains(&(cur.clone(), dep.clone())) {
                    continue;
                }
                if seen.insert(dep.clone()) {
                    stack.push(dep);
                }
            }
        }
        seen
    }
}

/// Run `cargo metadata` against the workspace and build the [`Workspace`] model.
pub fn load(repo_root: &Path) -> Result<Workspace, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let manifest = repo_root.join("Cargo.toml");
    let output = Command::new(&cargo)
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .map_err(|e| format!("failed to spawn `{cargo} metadata`: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "`cargo metadata` failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    let raw: RawMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("parse cargo metadata json: {e}"))?;

    let root = PathBuf::from(&raw.workspace_root);
    let ws_ids: HashSet<String> = raw.workspace_members.iter().cloned().collect();
    let id_to_name: HashMap<String, String> = raw
        .packages
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    let mut node_deps: HashMap<String, Vec<(String, bool)>> = HashMap::new();
    for node in &raw.resolve.nodes {
        let deps = node
            .deps
            .iter()
            .map(|d| {
                let dev_only = !d.dep_kinds.is_empty()
                    && d.dep_kinds.iter().all(|k| k.kind.as_deref() == Some("dev"));
                (d.pkg.clone(), dev_only)
            })
            .collect();
        node_deps.insert(node.id.clone(), deps);
    }

    let mut members: Vec<Member> = Vec::new();
    let mut name_to_id: HashMap<String, String> = HashMap::new();
    for pkg in &raw.packages {
        if !ws_ids.contains(&pkg.id) {
            continue;
        }
        let manifest_path = PathBuf::from(&pkg.manifest_path);
        let dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| manifest_path.clone());
        let rel_dir = dir
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| dir.to_string_lossy().to_string());
        let layer = classify(&rel_dir);
        let target_src_paths = pkg
            .targets
            .iter()
            .map(|t| PathBuf::from(&t.src_path))
            .collect();
        name_to_id.insert(pkg.name.clone(), pkg.id.clone());
        members.push(Member {
            name: pkg.name.clone(),
            src_dir: dir.join("src"),
            dir,
            rel_dir,
            layer,
            target_src_paths,
        });
    }
    members.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Workspace {
        root,
        members,
        id_to_name,
        ws_ids,
        name_to_id,
        node_deps,
    })
}
