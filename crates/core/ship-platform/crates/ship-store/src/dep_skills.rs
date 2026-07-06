//! Resolve dep skill refs from the package cache into [`Skill`] values.
//!
//! Dep skill refs look like `host.tld/owner/pkg/skill-name` (any git host).
//! Local refs (no hostname prefix) are left to the caller to handle.
//!
//! Resolution path:
//!   dep ref → split package path + within-package path
//!           → look up package in ship.lock → get `sha256:<hex>` hash
//!           → cache dir `~/.ship/cache/objects/<hex>/`
//!           → if `<within-path>/SKILL.md` exists → single skill
//!           → if `<within-path>/` is a namespace dir (sub-dirs with SKILL.md)
//!             → expand to all leaf skills within

use anyhow::{Context, Result};
use compiler::lockfile::{LockPackage, ShipLock};
use compiler::{Skill, SkillSource};
use std::path::{Path, PathBuf};

// ── Dep ref detection ─────────────────────────────────────────────────────────

/// Return `true` if `ref_str` is a dep skill ref (`host.tld/owner/pkg/…`).
///
/// A dep ref has a first slash-separated segment containing a dot (the hostname)
/// and at least two more slashes (owner + package + within-package path).
/// Local refs are plain skill ids such as `my-skill` or `review-pr`.
pub fn is_dep_ref(ref_str: &str) -> bool {
    let Some(first_slash) = ref_str.find('/') else {
        return false;
    };
    let host = &ref_str[..first_slash];
    // Host must contain a dot (e.g. github.com, git.example.com)
    if !host.contains('.') {
        return false;
    }
    // Need at least two more slashes after host: owner/pkg/within
    let after_host = &ref_str[first_slash + 1..];
    after_host.matches('/').count() >= 2
}

/// Parse a dep skill ref into `(package_path, within_package_path)`.
///
/// The package path is the first three slash-separated segments
/// (`host.tld/owner/pkg`). Everything after is the within-package path
/// (`skills/skill-name`).
///
/// Returns `None` if the ref has fewer than four segments (no within-package
/// path) or if the first segment contains no dot (not a hostname).
pub fn parse_dep_ref(ref_str: &str) -> Option<(&str, &str)> {
    // Split into segments: host / owner / pkg / within...
    let first_slash = ref_str.find('/')?;
    let host = &ref_str[..first_slash];
    if !host.contains('.') {
        return None;
    }
    // Find owner and pkg slashes
    let after_host = &ref_str[first_slash + 1..];
    let owner_slash = after_host.find('/')?;
    let after_owner = &after_host[owner_slash + 1..];
    let pkg_slash = after_owner.find('/')?;

    let pkg_end = first_slash + 1 + owner_slash + 1 + pkg_slash;
    let package_path = &ref_str[..pkg_end];
    let within_path = &ref_str[pkg_end + 1..];

    if within_path.is_empty() {
        return None;
    }

    Some((package_path, within_path))
}

// ── Cache-based resolution ────────────────────────────────────────────────────

/// Look up the cache hash for `package_path` in the given lockfile.
///
/// Returns the hex digest (without the `sha256:` prefix) on success.
/// Returns an error if the package is absent — the caller should surface this
/// as a cache-miss requiring `ship install`.
pub fn hash_from_lock(lock: &ShipLock, package_path: &str) -> Result<String> {
    let pkg: Option<&LockPackage> = lock.packages.iter().find(|p| p.path == package_path);

    let pkg = pkg.ok_or_else(|| {
        anyhow::anyhow!(
            "dependency {} not in cache — run ship install",
            package_path
        )
    })?;

    let hex = pkg
        .hash
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow::anyhow!("malformed hash '{}' for {}", pkg.hash, package_path))?
        .to_string();

    Ok(hex)
}

/// Build the filesystem path to a within-package path under the cache.
///
/// `cache_root` is `~/.ship/cache/` (or a test override).
/// Returns `<cache_root>/objects/<hex>/<within_path>`.
pub fn cache_skill_path(cache_root: &Path, hex: &str, within_path: &str) -> PathBuf {
    cache_root.join("objects").join(hex).join(within_path)
}

/// Resolve a dep skill ref to one or more [`Skill`] values.
///
/// If the ref points to a directory with a `SKILL.md`, returns a single skill.
/// If it points to a **namespace directory** (no `SKILL.md` but contains
/// sub-directories that each have `SKILL.md`), expands to all leaf skills.
///
/// `ref_str`    — full dep ref, e.g. `github.com/owner/pkg/better-auth`
/// `lock`       — parsed ship.lock
/// `cache_root` — `~/.ship/cache/`
pub fn resolve_dep_skill(ref_str: &str, lock: &ShipLock, cache_root: &Path) -> Result<Vec<Skill>> {
    let (package_path, within_path) = parse_dep_ref(ref_str)
        .ok_or_else(|| anyhow::anyhow!("invalid dep skill ref: '{}'", ref_str))?;

    let hex = hash_from_lock(lock, package_path)?;

    let target_dir = cache_skill_path(cache_root, &hex, within_path);
    let skill_md = target_dir.join("SKILL.md");

    // Direct skill — the ref points to a directory with SKILL.md.
    if skill_md.exists() {
        let raw = std::fs::read_to_string(&skill_md)
            .with_context(|| format!("reading cached skill file {}", skill_md.display()))?;
        return Ok(vec![parse_dep_skill(ref_str, &raw)]);
    }

    // Namespace expansion — the ref points to a directory containing sub-skills.
    if target_dir.is_dir() {
        let leaf_skills = find_leaf_skills(&target_dir);
        if !leaf_skills.is_empty() {
            let mut skills = Vec::new();
            for (sub_name, sub_path) in &leaf_skills {
                let raw = std::fs::read_to_string(sub_path)
                    .with_context(|| format!("reading {}", sub_path.display()))?;
                let sub_ref = format!("{}/{}", ref_str, sub_name);
                skills.push(parse_dep_skill(&sub_ref, &raw));
            }
            return Ok(skills);
        }
    }

    // Neither a skill nor a namespace — build a helpful error.
    let parent = Path::new(within_path).parent().unwrap_or(Path::new(""));
    let skills_dir = cache_skill_path(cache_root, &hex, &parent.to_string_lossy());
    let available = list_available_skills(&skills_dir);
    let available_str = if available.is_empty() {
        "(none found)".to_string()
    } else {
        available.join(", ")
    };
    anyhow::bail!(
        "dep skill '{}': path '{}' not found in cached package '{}'; \
         available skills: {}",
        ref_str,
        within_path,
        package_path,
        available_str
    );
}

/// Parse a SKILL.md from a dep package, using `dep_ref` as the skill id.
///
/// Delegates to the canonical [`compiler::skill_parser::parse_skill`] — one
/// parser for compile-path, dep-skill cache, and any future consumer. The
/// earlier hand-rolled parser only handled block-form `grants:` / `requires:`
/// and silently DROPPED inline `grants: [..]` / `requires: [..]` declarations,
/// so dep-loaded agents were missing half their declared capability surface
/// (#106). The canonical parser accepts BOTH forms.
fn parse_dep_skill(dep_ref: &str, raw: &str) -> Skill {
    let mut skill = compiler::skill_parser::parse_skill(dep_ref, raw);
    // Mark dep-loaded skills as Community so downstream consumers (the gate,
    // the SDK generator) can distinguish them from authored/builtin skills.
    skill.source = SkillSource::Community;
    skill
}

/// Find all leaf skill sub-directories within a namespace directory.
///
/// Returns `(sub_name, path_to_SKILL.md)` for each sub-directory that contains
/// a `SKILL.md`. Sorted by name for deterministic output.
fn find_leaf_skills(namespace_dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(namespace_dir) else {
        return vec![];
    };
    let mut results: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let skill_md = e.path().join("SKILL.md");
            if e.path().is_dir() && skill_md.exists() {
                Some((e.file_name().to_string_lossy().to_string(), skill_md))
            } else {
                None
            }
        })
        .collect();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

/// List skill names available in `skills_dir` (best-effort, for error messages).
fn list_available_skills(skills_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

// ── Batch resolution entry point ──────────────────────────────────────────────

/// Resolve all dep skill refs found in `skill_refs`, merging them with
/// `local_skills` (which remain unchanged).
///
/// - Refs without a hostname prefix are skipped (they are local).
/// - Refs already present by id in `local_skills` are skipped (no duplicates).
/// - A missing lock or cache hit causes a hard error with an actionable message.
///
/// `lock_path`  — `.ship/ship.lock`
/// `cache_root` — `~/.ship/cache/` (pass `None` to use the default)
pub fn resolve_dep_skills(
    skill_refs: &[String],
    local_skills: &[Skill],
    lock_path: &Path,
    cache_root: Option<&Path>,
) -> Result<Vec<Skill>> {
    // Collect only dep refs
    let dep_refs: Vec<&str> = skill_refs
        .iter()
        .filter(|r| is_dep_ref(r))
        .map(String::as_str)
        .collect();

    if dep_refs.is_empty() {
        return Ok(vec![]);
    }

    // Need the lockfile
    let lock = ShipLock::from_file(lock_path).with_context(|| {
        format!(
            "cannot read {} — run ship install to populate the lock file",
            lock_path.display()
        )
    })?;

    // Determine cache root
    let default_cache: PathBuf;
    let cache_root = match cache_root {
        Some(p) => p,
        None => {
            // Route through `ShipHome::from_env` so SHIP_HOME is honored and
            // the test-context isolation gate kicks in. A raw `home::home_dir()`
            // bypass here would let a non-isolated test resolve the operator's
            // real `~/.ship/cache/` — see datasafety-shiphome.
            default_cache = ship_substrate::ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache
        }
    };

    // Build existing id set to deduplicate
    let existing_ids: std::collections::HashSet<&str> =
        local_skills.iter().map(|s| s.id.as_str()).collect();

    let mut dep_skills = Vec::new();
    for ref_str in dep_refs {
        if existing_ids.contains(ref_str) {
            continue; // already present locally, skip
        }
        let skills = resolve_dep_skill(ref_str, &lock, cache_root)
            .with_context(|| format!("resolving dep skill '{}'", ref_str))?;
        for skill in skills {
            if !existing_ids.contains(skill.id.as_str()) {
                dep_skills.push(skill);
            }
        }
    }

    Ok(dep_skills)
}

/// Compose dep AGENTS into the consuming space (composition engine, #122).
///
/// Auto-compose model: each locked package's `exports.agents` are loaded from the
/// content-addressed cache and made available in the space. Each agent's id is
/// namespaced by the package path so dep agents never collide with local agents
/// or with each other. Mirrors `resolve_dep_skills`'s lock + cache resolution.
/// Returns an empty vec when there is no lockfile (no deps installed).
pub fn resolve_dep_agents(
    lock_path: &Path,
    cache_root: Option<&Path>,
) -> Result<Vec<compiler::AgentProfile>> {
    if !lock_path.exists() {
        return Ok(vec![]);
    }
    let lock =
        ShipLock::from_file(lock_path).with_context(|| format!("reading {}", lock_path.display()))?;

    let default_cache: PathBuf;
    let cache_root = match cache_root {
        Some(p) => p,
        None => {
            // See `resolve_dep_skills`: route through ShipHome so SHIP_HOME
            // and the test-context isolation gate are honored.
            default_cache = ship_substrate::ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache
        }
    };

    let mut out = Vec::new();
    for pkg in &lock.packages {
        let hex = pkg.hash.strip_prefix("sha256:").unwrap_or(&pkg.hash);
        let cache_dir = cache_root.join("objects").join(hex);
        let manifest_path = cache_dir.join("ship.jsonc");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = match compiler::manifest::ShipManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("dep agents: skipping {} — manifest parse: {e}", pkg.path);
                continue;
            }
        };
        for agent_rel in &manifest.exports.agents {
            let agent_path = cache_dir.join(agent_rel);
            let content = match std::fs::read_to_string(&agent_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("dep agents: {}::{agent_rel} unreadable: {e}", pkg.path);
                    continue;
                }
            };
            match compiler::jsonc::from_jsonc_str::<compiler::AgentProfile>(&content) {
                Ok(mut profile) => {
                    // Namespace by package path — collision-safe + deterministic.
                    profile.agent.id = format!("{}/{}", pkg.path, profile.agent.id);
                    out.push(profile);
                }
                Err(e) => tracing::warn!("dep agents: {}::{agent_rel} parse: {e}", pkg.path),
            }
        }
    }
    Ok(out)
}

/// Transitive capability surface of the installed dep tree (#119).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TransitiveCapabilities {
    /// Union of `requires` across every locked package's exported skills.
    pub required: Vec<String>,
    /// Union of `grants` across them.
    pub granted: Vec<String>,
    /// `required` not self-satisfied by `granted` — what the operator must
    /// approve at install (the supply-chain surface).
    pub unmet: Vec<String>,
}

/// Compute the transitive capability surface of the dep tree (#119): walk each
/// locked package's `exports.skills`, load them from the cache (reusing
/// `resolve_dep_skill`), and union their `requires`/`grants`. `unmet` is what
/// the tree demands that it doesn't grant itself. Surfaced at install for
/// approval. Returns empty when there is no lockfile.
pub fn transitive_capabilities(
    lock_path: &Path,
    cache_root: Option<&Path>,
) -> Result<TransitiveCapabilities> {
    if !lock_path.exists() {
        return Ok(TransitiveCapabilities::default());
    }
    let lock =
        ShipLock::from_file(lock_path).with_context(|| format!("reading {}", lock_path.display()))?;

    let default_cache: PathBuf;
    let cache_root = match cache_root {
        Some(p) => p,
        None => {
            // See `resolve_dep_skills`: route through ShipHome so SHIP_HOME
            // and the test-context isolation gate are honored.
            default_cache = ship_substrate::ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache
        }
    };

    let mut required: Vec<String> = Vec::new();
    let mut granted: Vec<String> = Vec::new();
    for pkg in &lock.packages {
        let hex = pkg.hash.strip_prefix("sha256:").unwrap_or(&pkg.hash);
        let manifest_path = cache_root.join("objects").join(hex).join("ship.jsonc");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = match compiler::manifest::ShipManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("transitive caps: skipping {} — manifest: {e}", pkg.path);
                continue;
            }
        };
        for skill_export in &manifest.exports.skills {
            let ref_str = format!("{}/{}", pkg.path, skill_export);
            match resolve_dep_skill(&ref_str, &lock, cache_root) {
                Ok(skills) => {
                    for s in skills {
                        required.extend(s.requires.iter().cloned());
                        granted.extend(s.grants.iter().cloned());
                    }
                }
                Err(e) => tracing::warn!("transitive caps: {ref_str}: {e}"),
            }
        }
    }
    required.sort();
    required.dedup();
    granted.sort();
    granted.dedup();
    let granted_set: std::collections::HashSet<&str> = granted.iter().map(String::as_str).collect();
    let unmet: Vec<String> = required
        .iter()
        .filter(|r| !granted_set.contains(r.as_str()))
        .cloned()
        .collect();
    Ok(TransitiveCapabilities { required, granted, unmet })
}

// ── Per-package composition bundle (Phase-3 J2) ──────────────────────────────

/// One locked package's contribution to the extension roll-up, loaded from the
/// content-addressed cache. Each `CachedExtension` is a self-contained bundle
/// the resolver turns into a single [`compiler::ExtensionDescriptor`].
///
/// The Phase-3 roll-up wants per-package data (each package becomes its own
/// extension descriptor so the shadow records can identify which package lost
/// on an id collision). The existing [`resolve_dep_agents`] /
/// [`resolve_dep_skills`] flatten every package's contribution into one vec —
/// fine for the compile path, lossy for the roll-up's provenance. This
/// function is the additive per-package surface; the existing flatteners stay
/// in place for the compile path (no motion).
/// Fields mirror the nouns the composition law composes per package:
/// `agent_profiles` (id namespaced by `pkg_path`), `skills` (id namespaced),
/// `views` (route/component_ref per the dep's `view.jsonc` sidecar,
/// `prefix` set to `pkg_path` so dep views mount under the package's own
/// namespace and never collide with the on-disk source or with other
/// packages — `compose_space`'s namespace-prefix merge keys on
/// `(prefix, route)`), and `content_hash` (the package's sha256 hex from
/// `ShipLock::hash`, the tier-level identity the shadow record carries on
/// collision).
#[derive(Debug, Clone)]
pub struct CachedExtension {
    /// The package's path in the lockfile, e.g. `github.com/owner/pkg`.
    pub pkg_path: String,
    /// The package's `sha256:<hex>` content hash from the lockfile — the
    /// cache tier's content hash the roll-up's cache key covers.
    pub content_hash: String,
    /// Agent profiles from the package's `exports.agents`, namespaced by
    /// `pkg_path` so dep agents never collide with the on-disk source or
    /// with other packages.
    pub agent_profiles: Vec<compiler::AgentProfile>,
    /// Skills from the package's `exports.skills`, namespaced by `pkg_path`.
    pub skills: Vec<compiler::Skill>,
    /// Views from the package's `exports.views`, each `prefix` set to
    /// `pkg_path`. The per-view data (`route`, `component_ref`) reads from
    /// `<view_dir>/view.jsonc` (the dep sidecar; absent → silently
    /// skipped, the same lenient contract the other noun loaders have).
    pub views: Vec<compiler::types::ViewDescriptor>,
}

/// Load every locked package's exports as a per-package bundle, suitable for
/// the Phase-3 roll-up's `ExtensionDescriptor` construction.
///
/// Returns an empty vec when there is no lockfile (no deps installed). Skips
/// packages whose `ship.jsonc` is missing or malformed (the same lenient
/// contract [`resolve_dep_agents`] has — bad packages get a `tracing::warn`
/// and the roll-up continues with the rest).
///
/// `lock_path`  — `.ship/ship.lock`
/// `cache_root` — `~/.ship/cache/` (pass `None` to use the default)
pub fn load_cached_extensions(
    lock_path: &Path,
    cache_root: Option<&Path>,
) -> Result<Vec<CachedExtension>> {
    if !lock_path.exists() {
        return Ok(vec![]);
    }
    let lock =
        ShipLock::from_file(lock_path).with_context(|| format!("reading {}", lock_path.display()))?;

    let default_cache: PathBuf;
    let cache_root = match cache_root {
        Some(p) => p,
        None => {
            // See `resolve_dep_skills`: route through ShipHome so SHIP_HOME is
            // honored and the test-context isolation gate kicks in.
            default_cache = ship_substrate::ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache
        }
    };

    let mut out = Vec::new();
    for pkg in &lock.packages {
        let hex = pkg.hash.strip_prefix("sha256:").unwrap_or(&pkg.hash).to_string();
        let cache_dir = cache_root.join("objects").join(&hex);
        let manifest_path = cache_dir.join("ship.jsonc");
        if !manifest_path.exists() {
            // No manifest in the cache — the package may be partially
            // installed; skip silently so the roll-up continues. A
            // follow-on `ship install` fills the gap.
            continue;
        }
        let manifest = match compiler::manifest::ShipManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "load_cached_extensions: skipping {} — manifest parse: {e}",
                    pkg.path
                );
                continue;
            }
        };

        let mut agent_profiles = Vec::new();
        for agent_rel in &manifest.exports.agents {
            let agent_path = cache_dir.join(agent_rel);
            let content = match std::fs::read_to_string(&agent_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        "load_cached_extensions: {}::{agent_rel} unreadable: {e}",
                        pkg.path
                    );
                    continue;
                }
            };
            match compiler::jsonc::from_jsonc_str::<compiler::AgentProfile>(&content) {
                Ok(mut profile) => {
                    profile.agent.id = format!("{}/{}", pkg.path, profile.agent.id);
                    agent_profiles.push(profile);
                }
                Err(e) => tracing::warn!(
                    "load_cached_extensions: {}::{agent_rel} parse: {e}",
                    pkg.path
                ),
            }
        }

        let mut skills = Vec::new();
        for skill_export in &manifest.exports.skills {
            let ref_str = format!("{}/{}", pkg.path, skill_export);
            match resolve_dep_skill(&ref_str, &lock, cache_root) {
                Ok(skilled) => skills.extend(skilled),
                Err(e) => tracing::warn!(
                    "load_cached_extensions: {ref_str}: {e}"
                ),
            }
        }

        // Views: each `exports.views` entry is a directory holding the
        // view's SUI template; the per-view data the roll-up needs
        // (`route`, `component_ref`) lives in a `view.jsonc` sidecar. The
        // `prefix` is set to `pkg.path` so dep views mount under the
        // package's own namespace and never collide with the on-disk
        // source or with other packages — `compose_space`'s namespace-
        // prefix merge keys on `(prefix, route)`. A sidecar that doesn't
        // exist is silently skipped (the same lenient contract agents +
        // skills have: a dep is partial-installable; missing sidecars
        // don't break the roll-up).
        let mut views = Vec::new();
        for view_rel in &manifest.exports.views {
            let sidecar_path = cache_dir.join(view_rel).join("view.jsonc");
            let content = match std::fs::read_to_string(&sidecar_path) {
                Ok(c) => c,
                Err(_) => {
                    // No sidecar — the view directory is the .sui bundle
                    // only; skip silently. The future on-disk view format
                    // is its own design pass.
                    continue;
                }
            };
            match compiler::jsonc::from_jsonc_str::<ViewSidecar>(&content) {
                Ok(sidecar) => {
                    views.push(compiler::types::ViewDescriptor {
                        prefix: pkg.path.clone(),
                        route: sidecar.route,
                        component_ref: sidecar.component_ref,
                    });
                }
                Err(e) => tracing::warn!(
                    "load_cached_extensions: {}::{}/view.jsonc parse: {e}",
                    pkg.path, view_rel
                ),
            }
        }

        out.push(CachedExtension {
            pkg_path: pkg.path.clone(),
            content_hash: hex,
            agent_profiles,
            skills,
            views,
        });
    }
    Ok(out)
}

/// The `<view_dir>/view.jsonc` sidecar a dep package ships to name a view
/// at composition time. Only the fields the composition law needs land
/// here — the bundle bytes (`.sui` template) are still read by deck at
/// render time, not at compose time.
#[derive(serde::Deserialize)]
struct ViewSidecar {
    /// The local route under the package's namespace (e.g. `/dashboard`).
    /// The full mount path is `prefix + route` per `compose_route_tree`.
    route: String,
    /// Which `.sui` bundle/element backs the route — a path/tag
    /// reference, not the bundle bytes.
    component_ref: String,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "dep_skills_tests.rs"]
mod tests;
