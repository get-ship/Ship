use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use crate::types::{
    AgentProfile, CollectionDef, HookConfig, McpServerConfig, Permissions, PipelineDef,
    PluginsManifest, SpaceManifest, Rule, Skill, TriggerDef, ViewDescriptor,
};

// ─── Phase-2 descriptor types ────────────────────────────────────────────────
// Project-shape extension: a TS source module resolved by the new resolver.
// `source` is the raw `.ts` text; `content_hash` is the canonical hex digest
// of that text (the exact algorithm is decided in the resolver crate — the
// descriptor carries it as an opaque string so the leaf stays hash-agnostic).
// `exports` is the list of named exports the resolver harvested at load time.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModuleDescriptor {
    pub path: PathBuf,
    pub source: String,
    pub content_hash: String,
    pub exports: Vec<String>,
}

// Phase-3 composition seam. The roll-up law consuming this is
// [`crate::compose::compose_space`]; the resolver's space orchestration
// builds the inputs and carries the composed result.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RollupInputs {
    /// Every extension in the space, in their declared order. Declared order
    /// breaks precedence-rank ties: later wins.
    pub extensions: Vec<ExtensionDescriptor>,
    /// The precedence rows the roll-up applies. Kind-level: every extension
    /// whose source kind matches a row composes at that row's rank; an
    /// extension with no matching row is a compose error, never a silent
    /// default. [`RollupInputs::default_precedence`] is the standing law.
    pub precedence: Vec<PrecedenceRow>,
}

impl RollupInputs {
    /// The standing composition law (census-phase3-agents.md Part 5, split
    /// per the format-migration Wave 2 census Part 3.4): cached packages <
    /// operator-enabled (user-global library) < space < per-agent overlay.
    /// Ranks are spaced so a future tier can slot between existing ones
    /// without renumbering.
    pub fn default_precedence() -> Vec<PrecedenceRow> {
        vec![
            PrecedenceRow { kind: ExtSourceKind::Cache, rank: 0 },
            PrecedenceRow { kind: ExtSourceKind::UserGlobal, rank: 500 },
            PrecedenceRow { kind: ExtSourceKind::Space, rank: 1000 },
            PrecedenceRow { kind: ExtSourceKind::Agent, rank: 2000 },
        ]
    }
}

/// One extension in the roll-up set (Phase 3).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtensionDescriptor {
    pub source: ExtSource,
    pub space_root: PathBuf,
    pub descriptor: ExtensionLibrary,
    pub content_hash: String,
}

/// Where an extension came from (Phase 3).
///
/// `Cache` (locked packages resolved from the `ship.lock`-pinned XDG cache)
/// and `UserGlobal` (operator-enabled extensions from `ShipHome::library_dir()`)
/// were originally one overloaded tier — split per the format-migration
/// Wave 2 census (Part 3.4) so cached packages and globally-enabled
/// extensions rank distinctly instead of sharing a tier.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtSource {
    Cache,
    UserGlobal,
    Space,
    Agent(String),
}

impl Default for ExtSource {
    fn default() -> Self {
        ExtSource::Space
    }
}

impl ExtSource {
    pub fn kind(&self) -> ExtSourceKind {
        match self {
            ExtSource::Cache => ExtSourceKind::Cache,
            ExtSource::UserGlobal => ExtSourceKind::UserGlobal,
            ExtSource::Space => ExtSourceKind::Space,
            ExtSource::Agent(_) => ExtSourceKind::Agent,
        }
    }
}

/// The kind of an [`ExtSource`] — the unit precedence rows rank at.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtSourceKind {
    Cache,
    UserGlobal,
    Space,
    Agent,
}

/// One row in the precedence table. Higher rank wins on id collision;
/// equal ranks resolve by declared extension order, later wins.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrecedenceRow {
    pub kind: ExtSourceKind,
    pub rank: u32,
}

/// Thin overrides that a feature branch can apply on top of project defaults.
/// Maps to the `[agent]` block in a feature's TOML frontmatter.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceOverrides {
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    /// Restrict to these MCP server IDs (empty = no restriction)
    pub mcp_servers: Vec<String>,
    /// Restrict to these skill IDs (empty = no restriction)
    pub skills: Vec<String>,
    /// Override harness list (empty = use project default)
    pub harnesses: Vec<String>,
}

/// Fully resolved, in-memory agent configuration.
///
/// Produced by [`resolve`] from pre-loaded project data.
/// No filesystem access occurs during resolution.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub harnesses: Vec<String>,
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    pub max_turns: Option<u32>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<Skill>,
    pub rules: Vec<Rule>,
    pub permissions: Permissions,
    pub hooks: Vec<HookConfig>,
    pub active_agent: Option<String>,
    /// Plugin install intent declared by the active preset.
    /// Passed through unchanged from `ExtensionLibrary::plugins`.
    pub plugins: PluginsManifest,
    /// Agent profiles parsed from `.ship/agents/*.toml`.
    /// Compiled into harness-native subagent definitions.
    pub agent_profiles: Vec<AgentProfile>,
    /// Environment variables to inject into harness settings.
    /// Source: `agents/env.toml` or `[env]` in preset TOML.
    /// Claude: `env` field in `.claude/settings.json`.
    pub env: std::collections::HashMap<String, String>,
    /// Restrict the model picker to these model IDs.
    /// Claude: `availableModels` in `.claude/settings.json`.
    pub available_models: Vec<String>,
    // ── Per-harness settings (typed deltas from the Ship-agent core) ───────────
    // Each is owned by its harness module; resolve() maps the flat library
    // fields into these. Dropping a harness = delete its module + one field.
    pub claude: crate::compile::ClaudeSettings,
    pub codex: crate::compile::CodexSettings,
    pub opencode: crate::compile::OpenCodeSettings,
    /// The capability grants the resolved skill set REQUESTS (the union of the
    /// auto-included capability-skills' `grants:`). A **request, not authority** —
    /// the daemon mints only the operator-approved subset; this is what an
    /// install/escalation approval is asked to confer. See
    /// `.artifacts/grant-resolution-arc-plan.md`.
    pub requested_grants: Vec<String>,
    /// Capabilities the resolved skills `require:` that no skill in the pool
    /// grants — an unsatisfiable request. Non-empty is a compile error the
    /// caller enforces (the compiler stays pure/infallible).
    pub unmet_capability_requirements: Vec<String>,
    /// Absolute path to the ship home (e.g. `/home/alice/.ship`). Injected
    /// by callers (CLI/daemon) that resolve via `ship_substrate::ShipHome`. The
    /// compiler is a pure transform and does not read environment variables —
    /// when this is `None`, the compiler emits an MCP entry without the
    /// `SHIP_HOME` env injection.
    pub ship_home_dir: Option<String>,
    /// Whether this config is compiled for a daemon-spawned, scoped launch
    /// (dispatch / command-bar / sandbox) rather than a manual `ship use`.
    /// Set true by the dispatcher/daemon, which inject `$SHIP_TOKEN` +
    /// `$SHIP_DAEMON_URL` into the agent's env. When true, the self-hosted
    /// `ship` server compiles to the daemon-hosted authed-HTTP entry
    /// (scope-enforced); when false (manual local dev), it stays the
    /// trusted-local stdio shim — the human is the host, full local access
    /// is fine and there is no spawner to provision the token env.
    pub scoped_launch: bool,
    // ── Phase-2 project-shape passthrough ─────────────────────────────────────
    /// Resolved TS modules from the multi-file `ship.config.ts` import graph. The
    /// Phase-0 leaf never produced this; Phase-2's resolver populates it on
    /// the TS path. The leaf's `resolve()` returns an empty vec — the
    /// `.jsonc` path doesn't have TS modules to surface.
    pub imported_modules: Vec<ModuleDescriptor>,
    /// Phase-3 composition seam. Phase 2 always carries `None`; Phase 3
    /// populates this with the per-space extension set when the space
    /// roll-up fn runs.
    pub rollup_inputs: Option<RollupInputs>,
}

/// Resolve the effective agent config from pre-loaded project data.
///
/// Resolution order (highest wins):
/// 1. Project defaults (manifest types + `agents/` content)
/// 2. Active mode filter (restricts servers/skills/rules)
/// 3. Feature overrides (model, harnesses, additional server/skill filter)
///
/// A self-contained library of agent config assets loaded from the `agents/`
/// directory. This is the primary input for the compiler in the new config model:
/// The manifest holds identity only; the library holds everything the compiler needs.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtensionLibrary {
    /// Agent definitions from `agents/*.jsonc`.
    #[serde(default)]
    pub agents: Vec<AgentProfile>,
    /// Active agent for this resolve (e.g. workspace override).
    #[serde(default)]
    pub active_agent: Option<String>,
    /// MCP server definitions from `mcp.jsonc`.
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    /// Skills from `skills/`.
    #[serde(default)]
    pub skills: Vec<Skill>,
    /// Rules from `rules/`.
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Pipeline definitions — from `pipelines/*.jsonc` today, from
    /// `definePipeline` declare-forms in `ship.config.ts` per format-migration
    /// Wave 3. Not yet cross-extension composed (`compose_space` passes
    /// the space tier's set through verbatim; see `compose.rs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pipelines: Vec<PipelineDef>,
    /// Trigger definitions — from `triggers/*.jsonc` today, from
    /// `defineTrigger` declare-forms in `ship.config.ts` per format-migration
    /// Wave 3. Not yet cross-extension composed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<TriggerDef>,
    /// Collection definitions — from `collections/*.jsonc` today, from
    /// `defineCollection` declare-forms in `ship.config.ts` per format-migration
    /// Wave 3. Not yet cross-extension composed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<CollectionDef>,
    /// View routes carried by this extension. Composed by namespace-prefix
    /// merge, not id-replace — see [`crate::compose::compose_space`]'s views
    /// handling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub views: Vec<ViewDescriptor>,
    /// Permissions from `agents/permissions.toml`.
    #[serde(default)]
    pub permissions: Permissions,
    /// Hooks from `agents/hooks.toml`.
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    /// Plugin install intent from the active preset's `[plugins]` section.
    #[serde(default)]
    pub plugins: PluginsManifest,
    /// Project-level harness defaults from `project.harness_defaults` in `ship.jsonc`.
    /// Merged as base under agent-level `*_settings_extra` during resolution.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub harness_defaults: std::collections::HashMap<String, serde_json::Value>,
    /// Pass-through from `[harness_settings.claude]` in the active preset.
    /// Merged verbatim into `.claude/settings.json` on compile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_settings_extra: Option<serde_json::Value>,
    /// Agent profiles from `.ship/agents/*.toml`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_profiles: Vec<AgentProfile>,
    /// Team agent files: `.ship/agents/teams/claude/<name>.md` → `.claude/agents/<name>.md`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_team_agents: Vec<(String, String)>,
    /// Environment variables: KEY=VALUE pairs injected into harness settings.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub env: std::collections::HashMap<String, String>,
    /// Restrict model picker to these IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub available_models: Vec<String>,
    /// Codex sandbox mode — translated key/value before emit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_sandbox: Option<String>,
    // ── Model ─────────────────────────────────────────────────────────────────
    /// Shared model override used by Claude, Codex, and OpenCode when no feature override present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    // ── Codex harness settings ────────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_approval_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_max_threads: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_max_depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_job_max_runtime_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_shell_env_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_notify: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_settings_extra: Option<serde_json::Value>,
    // ── OpenCode harness settings ────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opencode_settings_extra: Option<serde_json::Value>,
    // ── Legacy cursor fields (cursor harness removed; retained so existing
    //    project libraries / agent-def deserialize without error) ───────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_environment: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_settings_extra: Option<serde_json::Value>,
    // ── Claude harness settings ───────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_auto_updates: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_include_co_authored_by: Option<bool>,
    /// Project-scope `effortLevel` written to `.claude/settings.json`.
    /// When the active agent profile does NOT override via
    /// `harness_settings.claude.effort`, this is the value the compile
    /// emits. The dispatcher's `CompileOptions::default_effort` is the
    /// ultimate fallback (defaults to `Some("medium")` for job agents —
    /// the operator's user-scope `xhigh` never leaks into a dispatched
    /// session). `None` preserves the legacy "no override" behavior for
    /// the manual `ship use` path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_effort: Option<String>,
    /// Override the skills dir (project-relative path). `None` means use the
    /// default `.ship/skills/`. The `skills_dir` key in `ship.config.ts` lands here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_dir: Option<String>,
    /// Override the rules dir (project-relative path). `None` means use the
    /// default `.ship/rules/`. The `rules_dir` key in `ship.config.ts` lands here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_dir: Option<String>,
    /// Additional skill paths from `space.skill_paths` in `ship.jsonc`
    /// (migrated from agent-def's `skill_paths.rs`). Paths are project-relative;
    /// absolute paths are rejected at the call site. The first entry is always
    /// the default `.ship/skills/` (preserves today's behavior).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_paths: Vec<String>,
    /// Per-agent skills from the `agents/<id>/{SKILL.md,*.md}` convention walk
    /// (`req-fmt-agent-surface-convention`). BTreeMap so the resolved view is
    /// byte-stable across runs — the descriptor cache key is a content hash,
    /// but the in-memory form needs a stable order for diff/snapshot tests.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub per_agent_skills: BTreeMap<String, Vec<Skill>>,
    /// Resolved multi-file TS import graph: `ship.config.ts` + `agents.ts` + any
    /// per-domain modules. Populated by the Phase-2 resolver; the leaf
    /// `load_library` produces an empty vec.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imported_modules: Vec<ModuleDescriptor>,
    /// Injected globals baked into the SDK at resolve time (the same value
    /// `agent_config_eval::resolve_agents_with_globals` accepts). The
    /// leaf's `load_library` produces `Value::Null` — the `.jsonc` path
    /// doesn't have TS to inject globals into.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub globals: serde_json::Value,
    /// Phase-3 composition seam. Phase 2 always carries `None`; the resolver
    /// never populates this — Phase 3's roll-up fn does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollup_inputs: Option<RollupInputs>,
}

/// Merge harness_defaults (base) with *_settings_extra (agent override).
/// Returns the merged value, or None if both are None. Deep-merge logic is
/// delegated to [`crate::output::merge_json`] — single canonical impl.
fn merge_harness_extra(
    defaults: Option<&serde_json::Value>,
    agent_extra: Option<&serde_json::Value>,
) -> Option<serde_json::Value> {
    match (defaults, agent_extra) {
        (Some(d), Some(a)) => {
            let mut merged = d.clone();
            crate::output::merge_json(&mut merged, a);
            Some(merged)
        }
        (Some(d), None) => Some(d.clone()),
        (None, Some(a)) => Some(a.clone()),
        (None, None) => None,
    }
}

/// Resolve a [`ExtensionLibrary`] directly — the new-model entry point.
/// No `SpaceManifest` or filesystem access required.
pub fn resolve_library(
    library: &ExtensionLibrary,
    feature: Option<&WorkspaceOverrides>,
    active_agent_override: Option<&str>,
) -> ResolvedConfig {
    let config = SpaceManifest {
        agents: library.agents.clone(),
        mcp_servers: library.mcp_servers.clone(),
        ..Default::default()
    };
    // Library carries an active_agent — feed it into resolve as the override
    // when no explicit override was given. The compiler crate has no
    // implicit fallback; the input has to specify what's active.
    let active_agent_override = active_agent_override
        .map(str::to_string)
        .or_else(|| library.active_agent.clone());
    let mut resolved = resolve(
        &config,
        &library.skills,
        &library.rules,
        &library.permissions,
        &library.hooks,
        feature,
        active_agent_override.as_deref(),
    );
    resolved.plugins = library.plugins.clone();
    resolved.agent_profiles = library.agent_profiles.clone();
    resolved.env = library.env.clone();
    resolved.available_models = library.available_models.clone();
    // Phase 1A: model falls back to library.model when no feature/project override
    if resolved.model.is_none() {
        resolved.model = library.model.clone();
    }
    // Per-harness settings: map flat library fields into typed deltas.
    // harness_defaults is the base; agent-level *_settings_extra overrides.
    resolved.claude = crate::compile::ClaudeSettings {
        settings_extra: merge_harness_extra(
            library.harness_defaults.get("claude"),
            library.claude_settings_extra.as_ref(),
        ),
        team_agents: library.claude_team_agents.clone(),
        theme: library.claude_theme.clone(),
        auto_updates: library.claude_auto_updates,
        include_co_authored_by: library.claude_include_co_authored_by,
        effort: library.claude_effort.clone(),
    };
    resolved.codex = crate::compile::CodexSettings {
        sandbox: library.codex_sandbox.clone(),
        approval_policy: library.codex_approval_policy.clone(),
        reasoning_effort: library.codex_reasoning_effort.clone(),
        max_threads: library.codex_max_threads,
        max_depth: library.codex_max_depth,
        job_max_runtime_seconds: library.codex_job_max_runtime_seconds,
        shell_env_policy: library.codex_shell_env_policy.clone(),
        notify: library.codex_notify.clone(),
        settings_extra: merge_harness_extra(
            library.harness_defaults.get("codex"),
            library.codex_settings_extra.as_ref(),
        ),
    };
    resolved.opencode = crate::compile::OpenCodeSettings {
        settings_extra: merge_harness_extra(
            library.harness_defaults.get("opencode"),
            library.opencode_settings_extra.as_ref(),
        ),
    };
    // Phase-2: thread the project-shape passthroughs from the library into
    // the resolved view. Today these are sourced from the library verbatim;
    // future work (J3+) may project/filter before they reach a consumer.
    resolved.imported_modules = library.imported_modules.clone();
    resolved.rollup_inputs = library.rollup_inputs.clone();
    resolved
}

pub fn resolve(
    config: &SpaceManifest,
    skills: &[Skill],
    rules: &[Rule],
    permissions: &Permissions,
    hooks: &[HookConfig],
    feature: Option<&WorkspaceOverrides>,
    active_agent_override: Option<&str>,
) -> ResolvedConfig {
    // ── Active agent (resolved first — needed for harness target_agents) ─────
    let active_agent = active_agent_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter(|v| config.agents.iter().any(|a| a.agent.id == *v))
        .map(str::to_string);
    let agent = active_agent
        .as_deref()
        .and_then(|id| config.agents.iter().find(|a| a.agent.id == id));

    // ── Harnesses ─────────────────────────────────────────────────────────────
    // Resolution priority (highest wins):
    // 1. Feature/workspace explicit harnesses — propagate even if empty (unknown
    //    harnesses → empty rather than silent fallback).
    // 2. Project-level harnesses.
    // 3. Default: ["claude"].
    //
    // Agent profiles are harness-agnostic at the baseline. Per-agent overrides
    // of harness settings (e.g. model) live in `harness_settings.<harness>.<key>`,
    // not in selecting which harnesses to emit.
    let feature_has_explicit = feature.is_some_and(|f| !f.harnesses.is_empty());
    let feature_harnesses = feature
        .filter(|f| !f.harnesses.is_empty())
        .map(|f| normalize_harnesses(&f.harnesses))
        .unwrap_or_default();

    let harnesses = if feature_has_explicit {
        feature_harnesses
    } else {
        let project = normalize_harnesses(&config.harnesses);
        if project.is_empty() {
            vec!["claude".to_string()]
        } else {
            project
        }
    };

    // ── Model ─────────────────────────────────────────────────────────────────
    // Model comes from the feature/workspace override only — there's no
    // project-level default. The harness's CLI picks its own when none
    // is specified.
    let model = feature.and_then(|f| f.model.clone());

    // ── Cost / turns ──────────────────────────────────────────────────────────
    let max_cost_per_session = feature.and_then(|f| f.max_cost_per_session);
    let max_turns: Option<u32> = None;

    // ── MCP servers ───────────────────────────────────────────────────────────
    let mut mcp_servers = config.mcp_servers.clone();
    apply_agent_server_filter(&mut mcp_servers, agent);
    if let Some(f) = feature {
        apply_workspace_server_filter(&mut mcp_servers, f);
    }

    // ── Skills ────────────────────────────────────────────────────────────────
    let mut resolved_skills = skills.to_vec();
    apply_agent_skill_filter(&mut resolved_skills, agent);
    if let Some(f) = feature {
        apply_workspace_skill_filter(&mut resolved_skills, f);
    }
    // Auto-include capability-skills from the pool that satisfy `requires:`.
    // A `requires:` is a capability REQUEST; including the granting skill makes
    // the request resolvable. Conferring the grant is the operator's job — the
    // daemon mints only the approved subset of `requested_grants`.
    include_capability_skills(&mut resolved_skills, skills);
    let requested_grants = crate::types::skill::grants_from_skills(&resolved_skills);
    let unmet_capability_requirements = crate::types::skill::unmet_requirements(&resolved_skills);

    // ── Rules ─────────────────────────────────────────────────────────────────
    let mut resolved_rules = rules.to_vec();
    if let Some(m) = agent
        && !m.rules.refs.is_empty()
    {
        let allowed: HashSet<String> =
            m.rules.refs.iter().map(|id| normalize_rule_id(id)).collect();
        resolved_rules.retain(|rule| allowed.contains(&normalize_rule_id(&rule.file_name)));
    }

    ResolvedConfig {
        harnesses,
        model,
        max_cost_per_session,
        max_turns,
        mcp_servers,
        skills: resolved_skills,
        rules: resolved_rules,
        permissions: permissions.clone(),
        hooks: hooks.to_vec(),
        active_agent,
        plugins: PluginsManifest::default(),
        agent_profiles: Vec::new(),
        env: std::collections::HashMap::new(),
        available_models: Vec::new(),
        // Per-harness deltas default to empty here; resolve_library() fills
        // them from the flat ExtensionLibrary fields.
        claude: crate::compile::ClaudeSettings::default(),
        codex: crate::compile::CodexSettings::default(),
        opencode: crate::compile::OpenCodeSettings::default(),
        requested_grants,
        unmet_capability_requirements,
        ship_home_dir: None,
        scoped_launch: false,
        // Phase-2: the leaf `resolve()` has no ExtensionLibrary to source these
        // from. Defaults match the leaf's serde defaults so the round-trip is
        // stable; resolve_library() threads the library's values when present.
        imported_modules: Vec::new(),
        rollup_inputs: None,
    }
}

/// Auto-include capability-skills from `pool` that satisfy the resolved skills'
/// `requires:`. For each required capability not already granted by an included
/// skill, pull in a pool skill whose `grants:` covers it (capability-skills live
/// in the full pool even when not in the agent's own skill list). Capability-skills
/// confer grants, they don't `require:` — so a single pass over the request set
/// reaches fixpoint. A `requires:` with no granting skill anywhere stays unmet
/// (surfaced via `unmet_requirements`), never silently dropped.
fn include_capability_skills(resolved: &mut Vec<Skill>, pool: &[Skill]) {
    use std::collections::BTreeSet;
    let mut included: BTreeSet<String> = resolved.iter().map(|s| s.id.clone()).collect();
    let mut granted: BTreeSet<String> =
        resolved.iter().flat_map(|s| s.grants.iter().cloned()).collect();
    let required: BTreeSet<String> =
        resolved.iter().flat_map(|s| s.requires.iter().cloned()).collect();
    for cap in required.difference(&granted.clone()) {
        if let Some(skill) = pool
            .iter()
            .find(|s| !included.contains(&s.id) && s.grants.iter().any(|g| g == cap))
        {
            included.insert(skill.id.clone());
            for g in &skill.grants {
                granted.insert(g.clone());
            }
            resolved.push(skill.clone());
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn normalize_harnesses(ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in ids {
        let id = raw.trim().to_ascii_lowercase();
        if id.is_empty() {
            continue;
        }
        if !matches!(
            id.as_str(),
            "claude" | "codex" | "opencode" | "pi"
        ) {
            continue;
        }
        if seen.insert(id.clone()) {
            out.push(id);
        }
    }
    out
}

fn normalize_rule_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

fn apply_agent_server_filter(servers: &mut Vec<McpServerConfig>, agent: Option<&AgentProfile>) {
    if let Some(m) = agent
        && !m.mcp.servers.is_empty()
    {
        servers.retain(|s| m.mcp.servers.contains(&s.id));
    }
}

fn apply_workspace_server_filter(servers: &mut Vec<McpServerConfig>, feature: &WorkspaceOverrides) {
    if !feature.mcp_servers.is_empty() {
        let ids: Vec<&str> = feature.mcp_servers.iter().map(String::as_str).collect();
        servers.retain(|s| ids.contains(&s.id.as_str()));
    }
}

fn apply_agent_skill_filter(skills: &mut Vec<Skill>, agent: Option<&AgentProfile>) {
    if let Some(m) = agent
        && !m.skills.refs.is_empty()
    {
        // A skill ref that matches no pool skill used to be silently dropped by
        // the `retain` below — broken/mis-targeted agent->skill delivery was
        // invisible. Warn loudly (naming the ref + the consuming agent) before
        // filtering so every future unmatched ref is caught instead of vanishing.
        let agent_id = if m.agent.id.is_empty() {
            m.agent.name.as_str()
        } else {
            m.agent.id.as_str()
        };
        for skill_ref in &m.skills.refs {
            if !skills.iter().any(|s| &s.id == skill_ref) {
                eprintln!(
                    "warning: agent '{agent_id}' references skill '{skill_ref}' that matches no \
                     skill in the pool; the skill will not be delivered to this agent"
                );
            }
        }
        skills.retain(|s| m.skills.refs.contains(&s.id));
    }
}

fn apply_workspace_skill_filter(skills: &mut Vec<Skill>, feature: &WorkspaceOverrides) {
    if !feature.skills.is_empty() {
        let ids: Vec<&str> = feature.skills.iter().map(String::as_str).collect();
        skills.retain(|s| ids.contains(&s.id.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpaceManifest;
    use crate::types::agent_profile::{McpRefs, ProfileMeta, ProfileRules, SkillRefs};

    fn make_skill(id: &str) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: Default::default(),
            content: String::new(),
            source: Default::default(),
            vars: Default::default(),
            artifacts: vec![],
            grants: vec![],
            requires: vec![],
        }
    }

    fn make_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "cmd".to_string(),
            args: vec![],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: Default::default(),
            url: None,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        }
    }

    #[test]
    fn agent_skill_filter_drops_unmatched_ref() {
        // The agent references one real pool skill and one that matches nothing.
        // The real one is delivered; the unmatched ref is dropped (and a loud
        // warning is emitted to stderr naming the ref + agent — the regression
        // this guards, where the drop used to be silent).
        let agent = AgentProfile {
            agent: ProfileMeta {
                id: "shipwright".to_string(),
                name: "Shipwright".to_string(),
                ..Default::default()
            },
            skills: SkillRefs {
                refs: vec!["real-skill".to_string(), "ghost-skill".to_string()],
            },
            ..Default::default()
        };
        let mut skills = vec![make_skill("real-skill"), make_skill("other-skill")];
        apply_agent_skill_filter(&mut skills, Some(&agent));
        let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["real-skill"],
            "only the matched ref is delivered; the unmatched ref is dropped"
        );
    }

    #[test]
    fn agent_skill_filter_empty_refs_keeps_whole_pool() {
        // Empty refs apply no filter — the agent inherits the entire pool, and
        // no unmatched-ref warning fires.
        let agent = AgentProfile {
            agent: ProfileMeta {
                id: "rust-lane".to_string(),
                ..Default::default()
            },
            skills: SkillRefs { refs: vec![] },
            ..Default::default()
        };
        let mut skills = vec![make_skill("a"), make_skill("b")];
        apply_agent_skill_filter(&mut skills, Some(&agent));
        assert_eq!(skills.len(), 2, "empty refs apply no filter");
    }

    #[test]
    fn default_harness_is_claude() {
        let config = SpaceManifest::default();
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude"]);
    }

    #[test]
    fn requires_auto_includes_capability_skill_and_reports_requested() {
        // The agent declares only a workflow skill; that skill `requires:` a
        // capability granted by a capability-skill in the pool (not in the
        // agent's own list). Resolution auto-includes it and reports the request.
        let config = SpaceManifest {
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "lead".to_string(),
                    name: "Lead".to_string(),
                    ..Default::default()
                },
                skills: SkillRefs { refs: vec!["workflow".to_string()] },
                ..Default::default()
            }],
            ..Default::default()
        };
        let workflow = Skill {
            requires: vec!["ship-orchestration".to_string()],
            ..make_skill("workflow")
        };
        let cap = Skill {
            grants: vec!["ship-orchestration".to_string()],
            ..make_skill("ship-orchestration")
        };
        let skills = vec![workflow, cap];
        let r = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("lead"),
        );
        assert!(
            r.skills.iter().any(|s| s.id == "ship-orchestration"),
            "capability-skill must be auto-included to satisfy requires:"
        );
        assert_eq!(r.requested_grants, vec!["ship-orchestration".to_string()]);
        assert!(r.unmet_capability_requirements.is_empty());
    }

    #[test]
    fn unsatisfiable_requires_is_reported_unmet() {
        // No capability-skill grants what the workflow requires → unmet (a
        // compile error the caller enforces), and nothing is requested.
        let config = SpaceManifest {
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "lead".to_string(),
                    name: "Lead".to_string(),
                    ..Default::default()
                },
                skills: SkillRefs { refs: vec!["workflow".to_string()] },
                ..Default::default()
            }],
            ..Default::default()
        };
        let workflow = Skill {
            requires: vec!["ship-orchestration".to_string()],
            ..make_skill("workflow")
        };
        let r = resolve(
            &config,
            &[workflow],
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("lead"),
        );
        assert_eq!(
            r.unmet_capability_requirements,
            vec!["ship-orchestration".to_string()]
        );
        assert!(r.requested_grants.is_empty());
    }

    #[test]
    fn no_requires_means_no_requested_grants() {
        let r = resolve(
            &SpaceManifest::default(),
            &[make_skill("plain")],
            &[],
            &Permissions::default(),
            &[],
            None,
            None,
        );
        assert!(r.requested_grants.is_empty());
        assert!(r.unmet_capability_requirements.is_empty());
    }

    #[test]
    fn feature_overrides_harnesses() {
        let config = SpaceManifest {
            harnesses: vec!["claude".to_string()],
            ..Default::default()
        };
        let feature = WorkspaceOverrides {
            harnesses: vec!["codex".to_string()],
            ..Default::default()
        };
        let resolved = resolve(
            &config,
            &[],
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.harnesses, vec!["codex"]);
    }

    #[test]
    fn mode_filters_servers_and_skills() {
        let config = SpaceManifest {
            mcp_servers: vec![make_server("github"), make_server("linear")],
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "planning".to_string(),
                    name: "Planning".to_string(),
                    ..Default::default()
                },
                mcp: McpRefs { servers: vec!["github".to_string()] },
                skills: SkillRefs { refs: vec!["alpha".to_string()] },
                ..Default::default()
            }],
            ..Default::default()
        };
        let skills = vec![make_skill("alpha"), make_skill("beta")];
        let resolved = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("planning"),
        );
        assert_eq!(resolved.mcp_servers.len(), 1);
        assert_eq!(resolved.mcp_servers[0].id, "github");
        assert_eq!(resolved.skills.len(), 1);
        assert_eq!(resolved.skills[0].id, "alpha");
    }

    // ── Harness normalization ─────────────────────────────────────────────

    #[test]
    fn harnesses_normalized_to_lowercase() {
        let config = SpaceManifest {
            harnesses: vec!["CLAUDE".to_string(), "Codex".to_string()],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude", "codex"]);
    }

    #[test]
    fn duplicate_harnesses_deduplicated() {
        let config = SpaceManifest {
            harnesses: vec![
                "claude".to_string(),
                "Claude".to_string(),
                "CLAUDE".to_string(),
            ],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude"]);
    }

    #[test]
    fn unknown_harnesses_dropped() {
        let config = SpaceManifest {
            harnesses: vec!["claude".to_string(), "unknown-ai".to_string()],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude"]);
    }

    #[test]
    fn empty_and_whitespace_harnesses_dropped() {
        let config = SpaceManifest {
            harnesses: vec!["".to_string(), "  ".to_string(), "claude".to_string()],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude"]);
    }

    #[test]
    fn all_invalid_harnesses_falls_back_to_claude() {
        let config = SpaceManifest {
            harnesses: vec!["invalid".to_string(), "".to_string()],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.harnesses, vec!["claude"]);
    }

    // ── Agent target_agents is ignored for harness selection ─────────────
    // Agent profiles are harness-agnostic at the baseline; an agent's
    // target_agents (if non-empty for whatever legacy reason) does NOT
    // restrict which harnesses compile. Project list (and feature
    // override) are the only inputs to the harness loop.

    #[test]
    fn agent_target_agents_does_not_override_project_harnesses() {
        let config = SpaceManifest {
            harnesses: vec!["codex".to_string()],
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "any-agent".to_string(),
                    name: "Any Agent".to_string(),
                    ..Default::default()
                },
                target_agents: vec!["some-target".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, Some("any-agent"));
        assert_eq!(resolved.harnesses, vec!["codex"]);
    }

    // ── Rule normalization ────────────────────────────────────────────────

    #[test]
    fn mode_rules_strip_numeric_prefix_and_md_suffix() {
        let config = SpaceManifest {
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "strict".to_string(),
                    name: "Strict".to_string(),
                    ..Default::default()
                },
                rules: ProfileRules { refs: vec!["style".to_string()], ..Default::default() },
                ..Default::default()
            }],
            ..Default::default()
        };
        let rules = vec![
            Rule {
                file_name: "001-style.md".to_string(),
                content: "Style rules".to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            },
            Rule {
                file_name: "002-workflow.md".to_string(),
                content: "Workflow rules".to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            },
        ];
        let resolved = resolve(
            &config,
            &[],
            &rules,
            &Permissions::default(),
            &[],
            None,
            Some("strict"),
        );
        assert_eq!(resolved.rules.len(), 1);
        assert_eq!(resolved.rules[0].file_name, "001-style.md");
    }

    // ── Feature overrides for servers and skills ─────────────────────────

    #[test]
    fn feature_overrides_filter_servers() {
        let config = SpaceManifest {
            mcp_servers: vec![
                make_server("github"),
                make_server("linear"),
                make_server("slack"),
            ],
            ..Default::default()
        };
        let feature = WorkspaceOverrides {
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        };
        let resolved = resolve(
            &config,
            &[],
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.mcp_servers.len(), 1);
        assert_eq!(resolved.mcp_servers[0].id, "github");
    }

    #[test]
    fn feature_overrides_filter_skills() {
        let skills = vec![make_skill("alpha"), make_skill("beta"), make_skill("gamma")];
        let config = SpaceManifest::default();
        let feature = WorkspaceOverrides {
            skills: vec!["beta".to_string()],
            ..Default::default()
        };
        let resolved = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.skills.len(), 1);
        assert_eq!(resolved.skills[0].id, "beta");
    }

    #[test]
    fn empty_feature_filter_does_not_restrict() {
        let skills = vec![make_skill("alpha"), make_skill("beta")];
        let config = SpaceManifest {
            mcp_servers: vec![make_server("github")],
            ..Default::default()
        };
        let feature = WorkspaceOverrides::default(); // all empty
        let resolved = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.skills.len(), 2);
        assert_eq!(resolved.mcp_servers.len(), 1);
    }

    // ── Feature model override ──────────────────────────────────────────

    #[test]
    fn feature_model_overrides_project_model() {
        // SpaceManifest.ai was removed — there's no project-level model
        // anymore. Feature/workspace model override stands alone.
        let config = SpaceManifest::default();
        let feature = WorkspaceOverrides {
            model: Some("feature-model".to_string()),
            ..Default::default()
        };
        let resolved = resolve(
            &config,
            &[],
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.model.as_deref(), Some("feature-model"));
    }

    // ── Empty mode filter arrays ────────────────────────────────────────

    #[test]
    fn mode_with_empty_filters_passes_all_through() {
        let config = SpaceManifest {
            mcp_servers: vec![make_server("a"), make_server("b")],
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "open".to_string(),
                    name: "Open".to_string(),
                    ..Default::default()
                },
                // All filter arrays empty — should not restrict anything
                mcp: McpRefs { servers: vec![] },
                skills: SkillRefs { refs: vec![] },
                rules: ProfileRules { refs: vec![], ..Default::default() },
                ..Default::default()
            }],
            ..Default::default()
        };
        let skills = vec![make_skill("x"), make_skill("y")];
        let resolved = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            None,
        );
        assert_eq!(resolved.mcp_servers.len(), 2);
        assert_eq!(resolved.skills.len(), 2);
    }

    // ── Nonexistent agent override ignored ──────────────────────────────

    #[test]
    fn nonexistent_agent_override_yields_no_active() {
        // SpaceManifest no longer carries `active_agent` — that lives in
        // the `project_compile_state` projection, sourced from
        // `compile.completed` events. With no implicit fallback, a bogus
        // override resolves to "no active agent" rather than silently
        // selecting some project-level default.
        let config = SpaceManifest {
            agents: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "planning".to_string(),
                    name: "Planning".to_string(),
                    ..Default::default()
                },
                skills: SkillRefs { refs: vec!["plan-skill".to_string()] },
                ..Default::default()
            }],
            ..Default::default()
        };
        let skills = vec![make_skill("plan-skill"), make_skill("other")];
        let resolved = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("nonexistent"),
        );
        assert!(resolved.active_agent.is_none());
    }

    // ── resolve_library passthrough fields ─────────────────────────────

    #[test]
    fn resolve_library_passes_through_all_fields() {
        use crate::types::agent_profile::*;
        use crate::types::{PluginEntry, PluginsManifest};

        let library = super::ExtensionLibrary {
            plugins: PluginsManifest {
                install: vec![PluginEntry {
                    id: "test-plugin".to_string(),
                    harness: "claude".to_string(),
                }],
                scope: "project".to_string(),
            },
            claude_settings_extra: Some(serde_json::json!({"customKey": true})),
            agent_profiles: vec![AgentProfile {
                agent: ProfileMeta {
                    id: "test-agent".to_string(),
                    name: "Test Agent".to_string(),
                    version: None,
                    description: None,
                    role: None,
                    tags: vec![],
                    tools_preset: None,
                    active_tools: vec![],
                    icon: None,
                },
                capabilities: Vec::new(),
                skills: SkillRefs::default(),
                mcp: McpRefs::default(),
                plugins: PluginRefs::default(),
                permissions: ProfilePermissions::default(),
                rules: ProfileRules::default(),
                hooks: Default::default(),
                target_agents: Vec::new(),
                apps: Default::default(),
                harness_settings: Default::default(),
            }],
            claude_team_agents: vec![("team-lead.md".to_string(), "# Team Lead".to_string())],
            env: {
                let mut m = std::collections::HashMap::new();
                m.insert("KEY".to_string(), "VALUE".to_string());
                m
            },
            available_models: vec!["model-a".to_string()],
            ..Default::default()
        };

        let resolved = super::resolve_library(&library, None, None);

        assert_eq!(resolved.plugins.install.len(), 1);
        assert_eq!(resolved.plugins.install[0].id, "test-plugin");
        assert!(resolved.claude.settings_extra.is_some());
        assert_eq!(resolved.agent_profiles.len(), 1);
        assert_eq!(resolved.agent_profiles[0].agent.id, "test-agent");
        assert_eq!(resolved.claude.team_agents.len(), 1);
        assert_eq!(resolved.claude.team_agents[0].0, "team-lead.md");
        assert_eq!(resolved.env.get("KEY").map(String::as_str), Some("VALUE"));
        assert_eq!(resolved.available_models, vec!["model-a"]);
    }

    #[test]
    fn workspace_mode_override_takes_precedence() {
        let config = SpaceManifest {
            agents: vec![
                AgentProfile {
                    agent: ProfileMeta {
                        id: "planning".to_string(),
                        name: "Planning".to_string(),
                        ..Default::default()
                    },
                    skills: SkillRefs { refs: vec!["plan-skill".to_string()] },
                    ..Default::default()
                },
                AgentProfile {
                    agent: ProfileMeta {
                        id: "code".to_string(),
                        name: "Code".to_string(),
                        ..Default::default()
                    },
                    skills: SkillRefs { refs: vec!["code-skill".to_string()] },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let skills = vec![make_skill("plan-skill"), make_skill("code-skill")];

        let planning = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("planning"),
        );
        assert_eq!(planning.active_agent.as_deref(), Some("planning"));

        let code = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("code"),
        );
        assert_eq!(code.active_agent.as_deref(), Some("code"));
        assert_eq!(code.skills[0].id, "code-skill");
    }

    #[test]
    fn harness_defaults_merge_with_agent_extra() {
        use serde_json::json;

        // harness_defaults sets a base; agent extra overrides one key, adds another
        let mut defaults = std::collections::HashMap::new();
        defaults.insert(
            "claude".to_string(),
            json!({"theme": "dark", "autoUpdates": true, "nested": {"a": 1, "b": 2}}),
        );

        let library = ExtensionLibrary {
            harness_defaults: defaults,
            claude_settings_extra: Some(
                json!({"theme": "light", "newKey": "hello", "nested": {"b": 99, "c": 3}}),
            ),
            // codex: agent override but no defaults — agent wins
            codex_settings_extra: Some(json!({"policy": "suggest"})),
            ..Default::default()
        };

        let resolved = super::resolve_library(&library, None, None);

        // Claude: agent override wins on "theme", base kept for "autoUpdates", deep merge on "nested"
        let claude = resolved.claude.settings_extra.unwrap();
        assert_eq!(claude["theme"], "light");
        assert_eq!(claude["autoUpdates"], true);
        assert_eq!(claude["newKey"], "hello");
        assert_eq!(claude["nested"]["a"], 1);
        assert_eq!(claude["nested"]["b"], 99);
        assert_eq!(claude["nested"]["c"], 3);

        // Codex: agent override only, no defaults
        let codex = resolved.codex.settings_extra.unwrap();
        assert_eq!(codex["policy"], "suggest");
    }
}
