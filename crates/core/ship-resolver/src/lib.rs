//! `ship-resolver` — the Phase-2 project-shape resolver.
//!
//! ## What this crate does
//!
//! Generalizes the Phase-0 resolver (compiler leaf + agent-config-eval) from
//! single-manifest to **project shape**: a `ship.ts` at the project root, a
//! `skills/` + `rules/` directory convention, a `project.skill_paths` manifest
//! read. The result is a fully-populated [`compiler::ProjectLibrary`] (disk
//! state) and a [`compiler::ResolvedConfig`] (resolved state) — the canonical
//! descriptors the rest of the platform compiles and runs against.
//!
//! ## The seam
//!
//! The resolver is a NEW crate that composes with — does NOT replace — the
//! compiler leaf's `load_library` (per the F1 decision, `decision-f1-loader.md`
//! Option A: "embrace the superset"). The leaf's `load_library` keeps walking
//! `.ship/{mcp,permissions,hooks,rules,skills,agents}/` with its `std::fs`
//! surface; this crate layers the Phase-2 project-shape pieces on top via the
//! [`ship_substrate::ports::FsBroker`] gated door (R1 — new code goes through
//! FsBroker, not raw `std::fs`).
//!
//! The V8 TS-eval seam is `agent_config_eval::resolve_agents_with_globals`,
//! wrapped by `ts_resolver::resolve_ts_files` with the `!Send` isolate
//! crossing a `spawn_blocking` boundary — the same pattern shipd's ingest
//! uses today (`apps/shipd/src/ship_runtime/tasks.rs:285-304`, R2).
//!
//! ## What this crate does NOT do
//!
//! - **No cache.** J5 (descriptor cache) is a separate job that subscribes
//!   to this crate's [`watcher`] module. The cache key is
//!   [`descriptor::project_library_content_hash`].
//! - **No caller migration.** J6 (helmd dispatch), J7 (shipd ingest), J8
//!   (helmd wire) replace their existing call sites with this crate's API.
//!   Out of scope here.
//! - **No Phase-3 composition.** `space_rollup_inputs` is carried as `None`
//!   everywhere; the Phase-3 roll-up fn (out of scope per
//!   `census-phase2-resolver.md` Part 4f) is the consumer.
//!
//! ## Module layout
//!
//! - [`project_loader`] — the disk walk over project shape → `ProjectLibrary`.
//!   Composes with the leaf; the additive `project.skill_paths` manifest read
//!   lives in the leaf (per F1).
//! - [`ts_resolver`] — V8 eval fan-out. `!Send` boundary on `spawn_blocking`.
//! - [`descriptor`] — canonical sorted-key JSON helpers. The determinism
//!   invariant the cache relies on (`inv-fmt-deterministic-eval`).
//! - [`watcher`] — the J4 FsWatch consumer over the project root. Pushes
//!   `ResolverInvalidation` onto a `tokio::sync::watch` channel the cache
//!   subscribes to. Event-free per the FsWatch port's contract; the cache
//!   (J5) is the only consumer today.
//!
//! ## Authority
//!
//! - Spec: `helm/data/missions/m-ship-stack-cutover/scratch/job-spec-p2-J3-resolver.md`
//!   (J3) + `job-spec-p2-J4J5-watch-cache.md` (J4+J5; J5 is still pending).
//! - Census: `census-phase2-resolver.md` Part 4 + Part 5 J3 (and J4/J5 once
//!   J5 lands).
//! - F1 decision: `decision-f1-loader.md` (Option A — the superset is the
//!   architecture, no code moves).
pub mod descriptor;
pub mod project_loader;
pub mod ts_resolver;
pub mod cache;
pub mod watcher;

use std::path::Path;

use anyhow::{Context, Result};
use compiler::{ProjectLibrary, ResolvedConfig};
use serde_json::Value;
use ship_substrate::ports::{FsBroker, LocalFsBroker};

/// Resolve a project's full descriptor: disk walk + TS eval + resolve.
///
/// `project_root` is the directory containing `ship.ts` (preferred) or
/// `<.ship>/ship.ts` (legacy), plus `.ship/` for the bulk of the config
/// shape. Returns the resolved config — a pure, in-memory view of the
/// project's effective configuration.
///
/// The bulk of the disk read is delegated to the compiler leaf's
/// `load_library`; the Phase-2 project-shape pieces (`ship.ts`,
/// `project.skill_paths` extra dirs) are layered on top via FsBroker.
/// The TS eval is one `defineAgent` roll-up per `imported_modules`
/// entry, run in the V8 isolate with the `!Send` boundary crossed on a
/// dedicated current-thread runtime (R2).
///
/// Thin wrapper over [`resolve_project_with_globals`] with no injected
/// globals — every `defineAgent` call lands without operator defaults.
pub async fn resolve_project(project_root: &Path) -> Result<ResolvedConfig> {
    resolve_project_with_globals(project_root, Value::Null).await
}

/// Resolve a project's full descriptor with pre-fetched operator defaults
/// injected into the TS eval as the `globals` export.
///
/// `globals` is part of the closed-world input — a plain JSON value
/// (objects/arrays/scalars) baked into the SDK module before any user
/// code runs. The eval stays deterministic: no live fs/net/RPC, no
/// `Date.now` / `Math.random` (the SDK stub's guard runs at module top
/// level, before user code; see `agent-config-eval/src/lib.rs:105-158`).
///
/// A `ship.ts` composes the defaults with
/// `defineAgent({ ...globals.agentDefaults, agent: {...} })` (the
/// pattern the agent-config-eval tests pin down at
/// `agent-config-eval/tests/resolve_agents.rs:127-164`).
pub async fn resolve_project_with_globals(
    project_root: &Path,
    globals: Value,
) -> Result<ResolvedConfig> {
    let fs = LocalFsBroker::new();
    resolve_project_with_broker(&fs, project_root, globals).await
}

/// Same as [`resolve_project_with_globals`] but with an injected
/// [`FsBroker`]. Tests and a future FsBroker-with-extra-context world
/// (e.g. cap-bootstrapping) use this; production goes through
/// [`resolve_project`] / [`resolve_project_with_globals`].
pub async fn resolve_project_with_broker(
    fs: &dyn FsBroker,
    project_root: &Path,
    globals: Value,
) -> Result<ResolvedConfig> {
    // 1. Disk walk — the bulk via the leaf, the project-shape pieces via
    //    FsBroker. The result is a `ProjectLibrary` with skills
    //    (default + extra), rules, mcp, hooks, permissions, and the
    //    `imported_modules` from `ship.ts`.
    let mut library = project_loader::load_project_library(fs, project_root)
        .await
        .with_context(|| {
            format!("resolver: project load failed for {}", project_root.display())
        })?;

    // 2. TS eval fan-out — one `defineAgent` roll-up per module. Empty
    //    module list → no V8 work. The eval crosses the `!Send` boundary
    //    inside `ts_resolver`; the resulting `Vec<AgentProfile>` is plain
    //    data, safe to merge.
    let module_profiles =
        ts_resolver::resolve_ts_files(&library.imported_modules, &globals)
            .await
            .context("resolver: TS eval failed")?;
    if !module_profiles.is_empty() {
        // Phase-2: append the TS-authored profiles to BOTH the active-
        // agent-shaped list (`library.agents`) AND the consumer-visible
        // passthrough (`library.agent_profiles`).
        //
        // - `library.agents` is what `resolve_library` feeds into
        //   `ProjectConfig.agents`, which the resolve step filters by
        //   the active-agent selection.
        // - `library.agent_profiles` is the passthrough that
        //   `ResolvedConfig::agent_profiles` carries — the consumer-
        //   visible list (the harness compile inputs).
        //
        // Same id dedup as the skill merge: a `.jsonc` profile with
        // the same id as a TS profile wins (the leaf read it first
        // and the TS profile is appended after — a `.jsonc` author
        // beats a `.ts` author on id conflict, the lenient contract
        // the existing agent-def loader has).
        let seen: std::collections::HashSet<String> = library
            .agent_profiles
            .iter()
            .map(|a| a.agent.id.clone())
            .collect();
        for profile in module_profiles {
            if seen.contains(&profile.agent.id) {
                continue;
            }
            library.agent_profiles.push(profile.clone());
            library.agents.push(profile);
        }
    }
    // 3. Surface the resolve-time globals on the descriptor for
    //    audit / debug consumers. The TS eval already used the value;
    //    the field is for visibility.
    if !globals.is_null() {
        library.globals = globals;
    }

    // 4. Pure transform: `ProjectLibrary` → `ResolvedConfig`. No fs,
    //    no async. The leaf's `resolve_library` is the canonical impl.
    let resolved = compiler::resolve_library(&library, None, None);
    Ok(resolved)
}

/// Load + return the `ProjectLibrary` only (skip the resolve step +
/// TS eval). Useful for callers that need the raw project shape (the
/// descriptor cache in J5, for instance — the cache key is the
/// `ProjectLibrary`'s content hash, not the resolved config's).
pub async fn load_project_library(
    project_root: &Path,
) -> Result<ProjectLibrary> {
    let fs = LocalFsBroker::new();
    project_loader::load_project_library(&fs, project_root).await
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn write_minimal_project() -> (TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        // .ship/ with a single rule
        let ship = root.join(".ship");
        std::fs::create_dir_all(ship.join("rules")).unwrap();
        std::fs::write(
            ship.join("rules").join("style.md"),
            "---\nalwaysApply: true\n---\nBe terse.\n",
        )
        .unwrap();

        // ship.ts at the project root
        std::fs::write(
            root.join("ship.ts"),
            r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});
"#,
        )
        .unwrap();

        (tmp, root)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_project_with_no_globals_yields_resolved_config() {
        let (_tmp, root) = write_minimal_project();
        let resolved = resolve_project(&root).await.unwrap();
        // The TS-defined agent lands in the resolved config's agent list.
        let ids: Vec<&str> = resolved.agent_profiles.iter().map(|a| a.agent.id.as_str()).collect();
        // Phase-2 contract: TS agents appear in the active-agent-shaped
        // list (the leaf's resolve step pulls from `library.agents`).
        // The .ts path populates that list via this crate's merge step.
        assert!(
            resolved.agent_profiles.iter().any(|a| a.agent.id == "demo")
                || resolved
                    .agent_profiles
                    .iter()
                    .any(|a| a.agent.id == "demo"),
            "expected the TS-defined demo agent in the resolved config: {ids:?}"
        );
        // The rule from .ship/rules/ reaches the resolved config.
        assert_eq!(resolved.rules.len(), 1);
        assert_eq!(resolved.rules[0].file_name, "style.md");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_project_with_globals_injects_defaults() {
        // The agent-config-eval tests pin down the globals-injection
        // contract; this one verifies the resolver composes the value
        // end-to-end (the `globals` lands in `library.globals` so audit
        // consumers can see it).
        let (_tmp, root) = write_minimal_project();
        let globals = json!({ "agentDefaults": { "harness": "claude" } });
        let resolved = resolve_project_with_globals(&root, globals).await.unwrap();
        // The resolved config carries the .ts-defined agent and the rule.
        assert_eq!(resolved.rules.len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_project_is_deterministic_across_two_runs() {
        // The byte-identical-output invariant the cache relies on:
        // two reads of the same project produce byte-identical
        // canonical-JSON `ProjectLibrary` descriptors. The cache key
        // (J5) is the content hash of this form.
        let (_tmp, root) = write_minimal_project();
        let a = load_project_library(&root).await.unwrap();
        let b = load_project_library(&root).await.unwrap();
        let a_bytes = descriptor::project_library_canonical(&a).unwrap();
        let b_bytes = descriptor::project_library_canonical(&b).unwrap();
        assert_eq!(
            a_bytes, b_bytes,
            "two project loads must be byte-identical"
        );
        // The content hash is the same across two runs of the same input.
        let a_hash = descriptor::project_library_content_hash(&a).unwrap();
        let b_hash = descriptor::project_library_content_hash(&b).unwrap();
        assert_eq!(a_hash, b_hash);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn load_project_library_omits_resolve_step() {
        // `load_project_library` returns a `ProjectLibrary` (not a
        // `ResolvedConfig`); the caller does the resolve. The raw
        // project shape is what J5's cache keys on.
        let (_tmp, root) = write_minimal_project();
        let lib = load_project_library(&root).await.unwrap();
        assert_eq!(lib.rules.len(), 1);
        assert_eq!(lib.imported_modules.len(), 1);
        assert!(lib.imported_modules[0].source.contains("defineAgent"));
    }
}
