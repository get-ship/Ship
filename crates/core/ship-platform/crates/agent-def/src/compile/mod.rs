//! Compile service — the single shared compile path for the CLI and daemon.
//!
//! The daemon is the compile authority; the CLI is ambient. This service gives
//! native compile one shared home so the daemon can run it in-process.
//!
//! The compiler crate (transform) stays WASM-safe and has no platform deps.
//! This module owns the native-only loader (disk → `stack_model::ExtensionLibrary`)
//! and the full compile pipeline (library → written harness files).

pub mod agent;
pub mod loader;
pub mod permissions;
// `project_compile_state` projection — co-located with the compile service
// so a future crate-carve takes it along.
pub mod projection;
pub mod skill_parser;

#[cfg(test)]
mod tests;

pub use loader::{load_library, load_permission_preset, McpEntry, McpFile};
pub use permissions::PermissionPreset;
// `load_skills` (the agent-def disk-authoring loader) retired with
// Wave 5 — project skills arrive via `defineSkill` declare-forms in
// the space's `stack.config.ts` and land in `ExtensionLibrary::skills`
// through `ship_resolver::merge_ts_declarations`. The `SkillService`
// CRUD surface in `ship-agents` still writes `SKILL.md` files (the
// authoring path the Studio/db-scope gate owns; Phase-6's question) —
// the read side that compiled those files into the agent-def pipeline
// is gone.
pub use projection::{CompileStateProjection, ProjectCompileState};

use anyhow::{Context, Result};
use harness_config_compiler::CompileOutput;
use std::path::{Path, PathBuf};

/// Deliver each included sdk-skill's `references/docs/*.mdoc` pages beside the
/// `SKILL.md` the compiler emitted. The compiler writes the (templated)
/// `SKILL.md`; its TOC links to per-op reference pages that live in the embedded
/// sdk-skill tree — copy them into the same harness directory so the links
/// resolve in the compiled output. No-op for authored skills (no embedded refs).
fn augment_with_sdk_references(output: &mut CompileOutput) {
    let dirs: Vec<String> = output
        .skill_files
        .keys()
        .filter_map(|k| k.strip_suffix("/SKILL.md").map(str::to_string))
        .collect();
    for dir in dirs {
        let id = dir.rsplit('/').next().unwrap_or("");
        for (rel, content) in skill_parser::embedded_skill_reference_files(id) {
            output.skill_files.insert(format!("{dir}/{rel}"), content);
        }
    }
}

use ship_store::dep_skills::resolve_dep_skills;

use agent::{apply_agent_to_library, collect_all_skill_refs};

// ── Public types ─────────────────────────────────────────────────────────────

pub struct CompileOptions {
    /// Source root — the source directory on disk.
    pub space_root: PathBuf,
    /// Output root — where compiled files go. Defaults to `space_root`.
    pub output_root: Option<PathBuf>,
    /// Optionally restrict to a single harness.
    pub harness: Option<String>,
    /// Active agent id (already resolved from PathContext / ship.jsonc).
    pub active_agent: Option<String>,
    /// Extra skill IDs injected at compile time. Each must exist in the library.
    pub extra_skills: Vec<String>,
    /// Plugin IDs the active agent declared and that this compile installed.
    pub plugins_installed: Vec<String>,
    /// When true: resolve and compile but do not write files or emit
    /// `compile.completed`. Outputs are included in the returned summary so
    /// callers can dry-run-print them.
    pub dry_run: bool,
    /// Default `effortLevel` for the worktree's `.claude/settings.json`.
    /// Applied only when neither the active agent profile's
    /// `harness_settings.claude.effortLevel` nor the library's
    /// `claude_effort` is set. The dispatch path passes
    /// `Some("medium")` so a daemon-spawned agent never inherits the
    /// operator's user-scope `xhigh` default; the manual `ship use`
    /// path passes `None` and preserves the legacy no-override
    /// behavior. The agent profile's `harness_settings.claude.*`
    /// passthrough still wins on conflict (it's merged LAST into the
    /// settings patch).
    pub default_effort: Option<String>,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            space_root: PathBuf::new(),
            output_root: None,
            harness: None,
            active_agent: None,
            extra_skills: Vec::new(),
            plugins_installed: Vec::new(),
            dry_run: false,
            default_effort: None,
        }
    }
}
#[derive(Debug)]
pub struct CompileSummary {
    /// Capability grants the resolved skill set requests.
    pub requested_grants: Vec<String>,
    /// Harnesses that were compiled (or would be compiled in dry-run mode).
    pub harnesses: Vec<String>,
    /// Per-harness compile outputs. Populated even for dry runs so callers can
    /// print what would be written.
    pub outputs: Vec<(String, CompileOutput)>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn compile_project(opts: &CompileOptions) -> Result<CompileSummary> {
    // 1. Compose the on-disk source + locked cached packages + the
    //    operator-enabled library tier into a single `ComposedLibrary`
    //    (the Phase-3 roll-up). Post format-migration Wave 5 the leaf
    //    no longer reads `.ship/agents/*.jsonc` / `.ship/skills/<id>/SKILL.md`
    //    — those nouns only arrive via `stack.config.ts` declare-forms,
    //    harvested by the resolver's TS eval and merged into the
    //    composed library. The `#122` flat dep-agents merge
    //    (`ship_store::dep_skills::resolve_dep_agents`) is subsumed by
    //    `compose_space`'s precedence law (roll_up covers every case it
    //    did — the W2 regression test pinned that). The `library` the
    //    downstream compile consumes is the COMPOSED result, not the
    //    leaf's single-source read.
    let composed = ship_resolver::roll_up_registered_blocking(&opts.space_root, None)
        .context("compile_project: roll_up failed")?;
    compile_project_from_library(opts, composed.library)
}

/// Compile from an already-composed [`stack_model::ExtensionLibrary`] — the
/// Phase-3 roll-up path (`ship_resolver::roll_up` / `stack_model::compose_space`)
/// already merged the cache tier, the operator-enabled library tier, and the
/// space's own source under the precedence law, so the `#122` flat merge
/// ([`resolve_dep_agents`]) is skipped here — running it again would
/// double-add the same namespaced ids. `ship-code`'s dispatch-compile path
/// uses this so dispatch-compiled agents see the same composed world
/// shipd's boot index sees, instead of the single-source `.ship/` library
/// [`compile_project`] loads.
pub fn compile_project_from_library(
    opts: &CompileOptions,
    library: stack_model::ExtensionLibrary,
) -> Result<CompileSummary> {
    compile_from_library(opts, library)
}

fn compile_from_library(
    opts: &CompileOptions,
    mut library: stack_model::ExtensionLibrary,
) -> Result<CompileSummary> {
    let stack_root = opts.space_root.clone();
    let output_root = opts.output_root.as_deref().unwrap_or(&opts.space_root);

    // 2. Apply agent overrides (permissions, inline rules, harness list, plugins)
    if let Some(agent_id) = opts.active_agent.as_deref() {
        apply_agent_to_library(&mut library, agent_id, &opts.space_root)?;
    }
    library.active_agent = opts.active_agent.clone();

    // 3. Resolve dep skills from cache (github.com/ refs in ship.lock + cache)
    let all_skill_refs = collect_all_skill_refs(&library);
    if !all_skill_refs.is_empty() {
        // ship.lock: STACK ROOT beside stack.config.ts, one location.
        let lock_path = stack_root.join("ship.lock");
        let dep_skills = resolve_dep_skills(
            &all_skill_refs,
            &library.skills,
            &lock_path,
            None,
        )
        .context("resolving dep skills from cache")?;
        library.skills.extend(dep_skills);
    }

    // 4. Inject extra skills into the active agent's skill filter
    if !opts.extra_skills.is_empty() {
        for skill_id in &opts.extra_skills {
            anyhow::ensure!(
                library.skills.iter().any(|s| &s.id == skill_id),
                "skill not found: {} — check stack.config.ts defineSkill declare-forms",
                skill_id
            );
        }
        if let Some(agent_id) = opts.active_agent.as_deref()
            && let Some(agent) = library.agents.iter_mut().find(|a| a.agent.id == agent_id)
            && !agent.skills.refs.is_empty()
        {
            for skill_id in &opts.extra_skills {
                if !agent.skills.refs.contains(skill_id) {
                    agent.skills.refs.push(skill_id.clone());
                }
            }
        }
    }

    // 5. Resolve (agent filtering, harness selection)
    // Seed the project-scope `effortLevel` from `default_effort` only when
    // the library doesn't already carry an explicit value. The agent
    // profile's `harness_settings.claude.effortLevel` (passthrough) still
    // wins on conflict — it's merged LAST into the settings patch by
    // `build_claude_settings_patch`. Gap 2: a daemon-spawned compile
    // passes `Some("medium")` so a dispatched agent never inherits the
    // operator's user-scope `xhigh` default.
    if library.claude_effort.is_none() {
        library.claude_effort = opts.default_effort.clone();
    }
    let resolved =
        stack_model::resolve_library(&library, None, opts.active_agent.as_deref());
    if !resolved.unresolved_mcp_servers.is_empty() {
        anyhow::bail!(
            "unknown mcp server id(s) referenced by agent '{}': {}",
            opts.active_agent.as_deref().unwrap_or("<none>"),
            resolved.unresolved_mcp_servers.join(", ")
        );
    }
    let requested_grants = resolved.requested_grants.clone();

    // 6. Determine harnesses
    let harnesses: Vec<String> = match opts.harness.as_deref() {
        Some(h) => vec![h.to_string()],
        None => resolved.harnesses.clone(),
    };

    if harnesses.is_empty() {
        return Ok(CompileSummary {
            requested_grants,
            harnesses,
            outputs: vec![],
        });
    }

    // 7. Compile per harness
    let mut outputs = Vec::new();
    for harness_id in &harnesses {
        let Some(mut output) = harness_config_compiler::compile(&resolved, harness_id) else {
            eprintln!("warning: unknown harness '{}', skipping", harness_id);
            continue;
        };
        augment_with_sdk_references(&mut output);
        outputs.push((harness_id.clone(), output));
    }

    // 8. Write + post-compile steps (skipped in dry-run mode)
    if !opts.dry_run {
        for (harness_id, output) in &outputs {
            harness_config_compiler::write_output(output_root, harness_id, output)
                .with_context(|| format!("failed to write {} output", harness_id))?;
        }
        emit_compile_completed(
            &stack_root,
            opts.active_agent.as_deref(),
            &harnesses,
            &opts.plugins_installed,
        );
    }

    Ok(CompileSummary {
        requested_grants,
        harnesses,
        outputs,
    })
}

// ── Event emission ────────────────────────────────────────────────────────────

fn emit_compile_completed(
    stack_root: &Path,
    active_agent: Option<&str>,
    harnesses: &[String],
    plugins_installed: &[String],
) {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::types::{CompileCompleted, event_types::COMPILE_COMPLETED};
    use ship_substrate::persistence::event_store::SqliteEventStore;

    // The space id comes from the `projects` pointer. An unregistered project
    // records no compile state (space-less) — the daemon registers its
    // workspace at boot, so the normal flow always resolves.
    let Some(space_id) = ship_substrate::project::resolve_active_space(stack_root) else {
        tracing::warn!(
            stack_root = %stack_root.display(),
            "compile: workspace is not a registered space — compile.completed not recorded"
        );
        return;
    };
    let payload = CompileCompleted {
        active_agent: active_agent.map(str::to_string),
        compiled_at: chrono::Utc::now().to_rfc3339(),
        ship_version: env!("CARGO_PKG_VERSION").to_string(),
        harness: harnesses.first().cloned(),
        plugins_installed: plugins_installed.to_vec(),
    };
    let envelope = match EventEnvelope::new(COMPILE_COMPLETED, &space_id, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("compile: cannot build envelope: {e}");
            return;
        }
    };
    // compile.* is Space-scoped → write the bound space's space.db under the
    // global ship-home (the db compile_state::load(space_id) reads). `space_id`
    // is the scope key.
    let store = match ship_substrate::project::get_global_dir().and_then(|root| {
        SqliteEventStore::new_for_scope(
            &root,
            ship_types::event_registry::EventScope::Space,
            Some(space_id.as_str()),
        )
    }) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("compile: cannot open event store: {e}");
            return;
        }
    };
    // compile.completed is Space-scoped: the binding must be bound to this
    // space so enforce_scope stamps/validates the space key. A daemon-internal
    // Space binding (the compile pass runs in the daemon, no session) carries it.
    let space = ship_types::identity::SpaceId::new(space_id.as_str());
    let binding = ship_types::ScopeBinding::daemon_internal(
        Some(space.clone()),
        ship_types::Authority::Space { space_id: space },
    );
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        tracing::warn!("compile: cannot append compile.completed: {e}");
    }
}
