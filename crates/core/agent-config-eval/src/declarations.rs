//! Harvest every declare-form kind emitted by a `stack.config.ts` eval — not just
//! `defineAgent`. [`generated_stub::SDK_STUB`] tags each declare-form's
//! emitted value with a `__kind` discriminant (`{ __kind, value }`); this
//! module routes each tagged value to its noun's harvest shape.
//!
//! Two harvest strategies, per noun:
//!
//! - **Pass-through** (pipelines, triggers, collections): the stack authoring
//!   shape and `compiler`'s type are identical, so the emitted value
//!   deserializes directly into `ship_pipelines::PipelineDef` /
//!   `ship_triggers::TriggerDef` / `ship_collection::CollectionDef`.
//! - **Mapped** (mcp servers, config): `compiler`'s type predates the
//!   stack shape and carries fields the authoring shape doesn't — a private
//!   `*Declare` struct mirrors the stack shape exactly and a `into_*` fn
//!   maps it onto the compiler type, filling the gap with the same defaults
//!   a fresh `.jsonc` / `SKILL.md` entry would get.
//!
//! Slice 2c close-out: `defineSkill` and `defineRule` declare-forms die —
//! skills are on-disk `skills/<id>/SKILL.md` files (the resolver walks the
//! `defineConfig({ skills_dir })` dir), and rules arrive exclusively via
//! `defineAgent({ rules: { inline: "..." } })` (the inline block on the
//! agent composition; the on-disk `rules/<id>.md` convention died
//! with the rules-format cut, so the only authoring surface for
//! rules is the agent composition's inline block). The `RuleDeclare` /
//! `RuleFrontmatterDeclare` / `RuleAssetRefDeclare`
//! / `RuleGrantDeclare` / `RuleRequirementDeclare` types and their
//! `into_rule` harvest arms are gone. The corresponding `"rule"` case in
//! [`harvest`] is also gone: any `__kind` other than the surviving nouns
//! is rejected (the eval is treated as an error, same contract as before).
//!
//! Slice 2c also reshapes MCP server registration: `defineMcpServer`
//! declare-forms die; MCP server registrations move to
//! `defineConfig({ agents: { mcp: { <name>: {…} } } })` — same fields the
//! per-call form carried. The `McpServerDeclare` struct and its
//! `into_mcp_server_config` mapping live under `ConfigAgents` now; the
//! `"mcp_server"` arm of [`harvest`] is gone.
//!
//! Slice 3 close-out (A3+A4, 2026-07-11): hooks + permission presets leave
//! `.ship/hooks.jsonc` + `.ship/permissions.jsonc` (the leaf no longer
//! reads those files). Hooks arrive as on-disk files at
//! `<hooks_dir>/<event>.<ext>` (the resolver walks them; the leaf doesn't
//! touch the on-disk payload). Named permission presets arrive as
//! `defineConfig({ agents: { permissions: { presets: { ... } } } })` and
//! the resolver harvests the per-module map into
//! `ExtensionLibrary::permissions_presets`. No new declare-form kinds
//! land here — both moves are pure pointer-walk + manifest-block
//! additions to the existing `defineConfig` shape.
//!
//! Every mapped/pass-through noun's hand-written field set is checked
//! against [`generated_stub`]'s stack-derived fingerprints by
//! `drift_tests` below — the Rust-side half of the drift gate
//! `gen_eval_stub`'s `--check` opens at the TS-shape layer.

use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use agent_schema::{AgentProfile, McpServerConfig, McpServerType};
use ship_collection::CollectionDef;
use ship_pipelines::PipelineDef;
use ship_triggers::TriggerDef;
use ship_ui::ViewDescriptor;
use stack_model::manifest::ManifestDepValue;
use serde::Deserialize;
use serde_json::Value;

/// Every declare-form's harvested output from one `stack.config.ts` eval, merged
/// in call order. [`crate::resolve_agents_with_globals`] is a thin filter
/// over `.agents`; `ts_resolver::resolve_ts_files` merges these again
/// across multiple modules.
///
/// Slice 2c + rules-format cut: `skills` / `mcp_servers` are no longer
/// declared per-call from a `stack.config.ts`. Skills live on disk
/// (`skills/<id>/SKILL.md`); MCP servers live under
/// `defineConfig({ agents: { mcp: {…} } })` and are pushed into
/// `ExtensionLibrary::mcp_servers` by
/// `space_resolver::merge_ts_declarations`. Rules arrive exclusively via
/// the agent composition's `defineAgent({ rules: { inline } })` block
/// (the `harvest` step projects each agent's inline body into
/// `declarations.rules`; the on-disk `rules/<id>.md` convention died
/// with the rules-format cut). The fields stay on the struct (typed
/// `Vec<…>`) so the canonical-JSON surface remains shape-stable
/// for the descriptor cache's content hash.
#[derive(Debug, Clone, Default)]
pub struct Declarations {
    pub agents: Vec<AgentProfile>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<agent_schema::Skill>,
    pub rules: Vec<agent_schema::Rule>,
    pub pipelines: Vec<PipelineDef>,
    pub triggers: Vec<TriggerDef>,
    pub collections: Vec<CollectionDef>,
    /// Views harvested from `defineView` calls. The composition law
    /// (`stack_model::compose::compose_space`) merges these by namespace-prefix
    /// merge, not id-replace — see `ViewDescriptor` and the phase-4 J1/J2
    /// roll-up. Hand-spec'd shape (per `defineRule` precedent): no
    /// `@ship/stack` counterpart yet (`ViewDescriptor` is provisional
    /// pending sail/deck alignment, `decision-views-deck-boundary.md`).
    pub views: Vec<ViewDescriptor>,
    /// Per-extension-project options harvested from `defineConfig` calls.
    /// Zero-or-one entry per module (the function emits a `"config"` value
    /// the same way the other declare-forms do; the user-facing
    /// `export default defineConfig({...})` shape is purely for ecosystem
    /// conformance). The resolver reads `declarations.config` after the
    /// fan-out and threads each per-module value into the right
    /// `ExtensionLibrary` field (skills from the on-disk walk override
    /// `skill_paths`; the `agents.mcp` map seeds `mcp_servers`).
    /// Slice 2c adds `agents_dir` / `collections_dir` / `pipelines_dir` /
    /// `triggers_dir` and the `agents.mcp` block to the shape. Rules
    /// don't appear in this struct: the rules-format cut retired the
    /// on-disk `rules/<id>.md` convention — rule bodies arrive
    /// exclusively via the agent composition's inline
    /// `rules: { inline: "..." }` block (the `harvest` step projects
    /// each agent's body into `declarations.rules`).
    pub config: Vec<ConfigDeclare>,
}

/// The `{ __kind, value }` shape every declare-form's collector
/// (`__emit` in [`generated_stub::SDK_STUB`]) wraps its config in.
#[derive(Debug, Deserialize)]
struct Envelope {
    __kind: String,
    value: Value,
}

/// Route one emitted value (call index `index`, for error messages) into
/// `out`. Fails loudly on an unrecognized kind or a value that doesn't
/// match its noun's expected shape — a declare-form call is either well-
/// formed or the eval is treated as an error, the same contract
/// the slice-2c close-out keeps.
pub fn harvest(out: &mut Declarations, index: usize, raw: Value) -> Result<()> {
    let envelope: Envelope = serde_json::from_value(raw)
        .with_context(|| format!("declare-form call #{index} did not emit a tagged declaration"))?;
    match envelope.__kind.as_str() {
        // ARC A5: agent nouns are JSONC/on-disk only. defineAgent is
        // rejected so there is no silent dual path with agents/*.jsonc.
        "agent" => bail!(
            "declare-form call #{index}: `defineAgent` is retired (ARC A5) — author agents \
             as `agents/<id>.jsonc` (JSONC AgentProfile); space-resolver loads the plain dir"
        ),
        "pipeline" => {
            let def: PipelineDef = serde_json::from_value(envelope.value).with_context(|| {
                format!("definePipeline call #{index} did not match PipelineDef")
            })?;
            out.pipelines.push(def);
        }
        "trigger" => {
            let def: TriggerDef = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineTrigger call #{index} did not match TriggerDef"))?;
            out.triggers.push(def);
        }
        "collection" => {
            let def: CollectionDef = serde_json::from_value(envelope.value).with_context(|| {
                format!("defineCollection call #{index} did not match the collection shape")
            })?;
            out.collections.push(def);
        }
        "view" => {
            // The `defineView` value is the compiler's `ViewDescriptor` shape
            // (route/prefix/component_ref) — a pass-through, mirroring
            // `definePipeline` / `defineCollection`. The composition law
            // (`stack_model::compose::compose_space`) merges the resulting
            // `out.views` by namespace-prefix merge.
            let def: ViewDescriptor =
                serde_json::from_value(envelope.value).with_context(|| {
                    format!("defineView call #{index} did not match ViewDescriptor")
                })?;
            out.views.push(def);
        }
        "config" => {
            // The `defineConfig` value carries the per-extension-project
            // `skills_dir` (single dir name, no trailing slash, relative
            // to the dir containing the `stack.config.ts` declaring it) +
            // the slice-2c additions: `agents_dir`, `collections_dir`,
            // `pipelines_dir`, `triggers_dir`, and the `agents.mcp` block
            // (the MCP server registrations; the per-call
            // `defineMcpServer` declare-form died with slice 2c).
            // Hand-spec'd (no `@ship/stack` counterpart yet) — the same
            // posture as `defineView` (the `defineRule` declare-form also
            // died with slice 2c, so the on-disk `rules/<id>.md`
            // convention likewise retired — rules arrive exclusively via
            // the agent composition's `rules: { inline: "..." }` block).
            // The resolver's `merge_ts_declarations` walks `out.config`
            // and threads each value into the right `ExtensionLibrary`
            // field.

            let declared: ConfigDeclare =
                serde_json::from_value(envelope.value).with_context(|| {
                    format!("defineConfig call #{index} did not match the config shape")
                })?;
            out.config.push(declared);
        }
        // Slice 2c close-out: the dead declare-form kinds. `defineSkill`
        // is gone (skills are on-disk files, walked from
        // `defineConfig.skills_dir`); `defineRule` is gone (rules arrive
        // exclusively via `defineAgent({ rules: { inline: "..." } })` —
        // the on-disk `rules/<id>.md` convention died with the
        // rules-format cut); the per-call `defineMcpServer` is gone
        // (MCP registrations live under `defineConfig.agents.mcp`).
        // Any of these `__kind` values landing here is a bug — a
        // re-introduction or a stale module — and the eval is treated
        // as an error, same as an unknown kind.
        "skill" => bail!(
            "declare-form call #{index}: `defineSkill` is retired (ARC A5) — author skills \
             on-disk at `<skills_dir>/<id>/SKILL.md`; the resolver walks the dir"
        ),
        "rule" => bail!(
            "declare-form call #{index}: `defineRule` is retired (ARC A5) — author rules as \
             `rules/<id>.md` (or agent-doc `rules.inline` in agents/<id>.jsonc)"
        ),
        "mcp_server" => bail!(
            "declare-form call #{index}: `defineMcpServer` is retired (ARC A5) — author MCP \
             servers in `mcp.jsonc` (`mcp.servers` map)"
        ),
        other => bail!("declare-form call #{index}: unrecognized kind `{other}`"),
    }
    Ok(())
}

// ─── MCP server declare shape (slice 2c) ─────────────────────────────────────
//
// The `McpServerDef` authoring shape (`@ship/stack/mcp-servers.ts`). Lives
// here now because the declare-form it belonged to (`defineMcpServer`) died
// with slice 2c: MCP server registrations move to
// `defineConfig({ agents: { mcp: { <name>: McpServerDeclare } } })`. The
// `ConfigAgents::mcp` map is the harvest surface; the resolver walks it
// (in `space_resolver::merge_ts_declarations`) and converts each entry via
// `McpServerDeclare::into_mcp_server_config` (the same mapping the dead
// per-call form used). Same fields, same defaults — only the route in
// changed.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerDeclare {
    pub id: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub scope: String,
}
/// Stack-fingerprinted field sets — only consumed by the `drift_tests`
/// module (asserts the Rust harvest matches the generated stub fingerprints).
/// `#[allow(dead_code)]` keeps the lib build green; the constants are part
/// of the agent-config-eval public API surface.
#[allow(dead_code)]
pub const MCP_SERVER_DECLARE_FIELDS: &[&str] = &[
    "args",
    "command",
    "env",
    "id",
    "scope",
    "server_type",
    "url",
];
impl McpServerDeclare {
    /// Map onto `stack_model::McpServerConfig`. The authoring shape has no
    /// separate display name (only `id`) — `name` doubles as `id`, same
    /// as `defineAgent`'s `id`/`name` collapse when a config omits one.
    /// `disabled`/`timeout_secs`/the Codex+Cursor per-server fields have
    /// no TS-authored equivalent yet; they default off, same as a fresh
    /// `mcp.jsonc` entry would.
    pub fn into_mcp_server_config(self) -> McpServerConfig {
        let server_type = match self.server_type.as_str() {
            "http" => McpServerType::Http,
            "sse" => McpServerType::Sse,
            _ => McpServerType::Stdio,
        };
        McpServerConfig {
            id: self.id.clone(),
            name: self.id,
            command: self.command.unwrap_or_default(),
            args: self.args.unwrap_or_default(),
            env: self.env.unwrap_or_default(),
            scope: self.scope,
            server_type,
            url: self.url,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: Vec::new(),
            codex_disabled_tools: Vec::new(),
            cursor_env_file: None,
        }
    }
}

// ─── Per-extension-project config (defineConfig) ──────────────────────────
//
// The `defineConfig({ … })` value (a per-extension-project authoring shape,
// declared as `export default defineConfig({...})` from each extension's
// `stack.config.ts` for ecosystem conformance). The Rust harvest pushes one
// entry per `defineConfig` call into `Declarations::config`; the resolver's
// `merge_ts_declarations` walks that vec and threads each per-module value
// into the matching `ExtensionLibrary` field.
//
// Slice 2c additions:
//   - `agents_dir` / `collections_dir` / `pipelines_dir` / `triggers_dir`:
//     the per-extension-project dir pointers for the agent / collection /
//     pipeline / trigger source dirs. The default is the dir name beside
//     the `stack.config.ts` (`agents/`, `collections/`, `pipelines/`,
//     `triggers/`). The space_loader enumerates these dirs' `*.ts` files
//     and appends them as `ModuleDescriptor`s into `imported_modules` —
//     the resolver's `resolve_ts_files` already iterates the vec, so the
//     fan-out is "enumeration feeding existing machinery" (the job
//     spec's framing). Per-extension-project overrides via `defineConfig`
//     are read here; the spec mandates the pointer exists, but the
//     ship repo's own manifest uses the defaults (a later job can plumb
//     the override if a non-default layout ever ships).
//   - `agents.mcp`: the MCP server registration block. The per-call
//     `defineMcpServer` declare-form died with slice 2c; registrations
//     live here as a `name → McpServerDeclare` map. The
//     `merge_ts_declarations` walk converts each entry via
//     `McpServerDeclare::into_mcp_server_config` and seeds
//     `library.mcp_servers` with the same id-collision policy as the
//     dead per-call form (first wins).
//
// All scalar dir values are SINGLE dir names (no trailing slash), relative
// to the dir containing the `stack.config.ts` declaring them. The last
// `defineConfig` wins for each scalar (root-last-wins). An empty
// `defineConfig({})` is valid — the resolver sees an empty `ConfigDeclare`
// and falls through to the unified defaults.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConfigDeclare {
    /// The dir name where on-disk skills live (default `"skills/"`).
    #[serde(default)]
    pub skills_dir: Option<String>,
    /// The dir name where on-disk agent source `.ts` files live
    /// (default `"agents/"`).
    #[serde(default)]
    pub agents_dir: Option<String>,
    /// The dir name where on-disk collection source `.ts` files live
    /// (default `"collections/"`).
    #[serde(default)]
    pub collections_dir: Option<String>,
    /// The dir name where on-disk pipeline source `.ts` files live
    /// (default `"pipelines/"`).
    #[serde(default)]
    pub pipelines_dir: Option<String>,
    /// The dir name where on-disk trigger source `.ts` files live
    /// (default `"triggers/"`).
    #[serde(default)]
    pub triggers_dir: Option<String>,
    /// The dir name where on-disk hook payload files live
    /// (default `"hooks/"`). Each file at
    /// `<hooks_dir>/<event-stem>.<ext>` becomes a `HookConfig` whose
    /// `id = <event-stem>`, `trigger = <event>`, and
    /// `command = "bash <hooks_dir>/<event-stem>.<ext>"`. The resolver
    /// walks the dir the same way it walks skills + rules. The
    /// supported `<event-stem>` values mirror the
    /// [`stack_model::HookTrigger`] enum members exactly:
    /// `pre-tool-use` / `post-tool-use` / `notification` / `stop` /
    /// `subagent-stop` / `pre-compact`. Slice 3 close-out: the leaf
    /// no longer reads `.ship/hooks.jsonc` (the file died with this
    /// slice).
    #[serde(default)]
    pub hooks_dir: Option<String>,
    /// Space dependency declarations. Slice 5 moves the space's own
    /// `dependencies` block here from `.ship/ship.jsonc`; cache-object
    /// package manifests still use `ship.jsonc` until the registry story.
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, ManifestDepValue>,
    /// Per-namespace config blocks. Slice 2c: the `agents.mcp` block —
    /// the only `agents.*` sub-block today. The shape mirrors
    /// `@ship/stack`'s `ConfigAgents` (which doesn't exist yet — this is
    /// the per-extension-project authoring surface, hand-spec'd like
    /// `defineRule` / `defineView`). A future wave may add
    /// `agents.defaults`, etc.
    #[serde(default)]
    pub agents: Option<ConfigAgents>,
}

/// The `defineConfig({ agents: { … } })` block. Sub-blocks:
/// - `mcp` — the MCP server registration map (slice 2c).
/// - `permissions` — the named permission preset map (slice 3
///   close-out, replaces the per-preset sections of the dead
///   `.ship/permissions.jsonc`).
/// `None` defaults to "no agents-side config contributed from this
/// module's config" (a module without an `agents` block contributes
/// nothing to `ExtensionLibrary::mcp_servers` /
/// `ExtensionLibrary::permissions_presets`).
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConfigAgents {
    /// MCP server registrations. Keyed by server id (matching the
    /// `McpServerDeclare::id` field — the same identifier the
    /// per-call `defineMcpServer` form used). `merge_ts_declarations`
    /// walks this map and pushes each entry into
    /// `ExtensionLibrary::mcp_servers` via
    /// `McpServerDeclare::into_mcp_server_config`.
    #[serde(default)]
    pub mcp: HashMap<String, McpServerDeclare>,
    /// Named permission presets. Keyed by preset name (e.g.
    /// `"ship-readonly"` / `"ship-standard"`). Each entry is a
    /// [`PermissionPresetDeclare`] that the resolver converts
    /// 1:1 into a [`stack_model::PermissionPreset`]
    /// (same field set — the Declare type exists so the schema can
    /// keep serde's strict-shape discipline without exposing
    /// `compiler`'s types in the public harvest API).
    /// `merge_ts_declarations` walks this map and seeds
    /// `ExtensionLibrary::permissions_presets` with the same
    /// id-collision policy as `agents.mcp` (first wins).
    /// Slice 3 close-out.
    #[serde(default)]
    pub permissions: Option<ConfigPermissions>,
}

/// The `defineConfig({ agents: { permissions: { ... } } })` block. The
/// only sub-block is `presets` — the named permission preset map
/// (slice 3 close-out). `None` defaults to "no permission presets
/// contributed from this module's config" (a module without an
/// `agents.permissions` block contributes nothing to
/// `ExtensionLibrary::permissions_presets`).
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConfigPermissions {
    /// Permission preset definitions. Keyed by preset name. The
    /// per-preset shape matches `schemas/permissions.schema.json`'s
    /// `PermissionPreset` (`default_mode`, `tools_allow` /
    /// `tools_deny` / `tools_ask`, `additional_directories`). The
    /// resolver converts each entry into a
    /// `stack_model::PermissionPreset` and seeds
    /// `ExtensionLibrary::permissions_presets`.
    #[serde(default)]
    pub presets: HashMap<String, PermissionPresetDeclare>,
}

/// The `defineConfig({ agents: { permissions: { presets: { <name>: ... } } } })`
/// per-preset shape. Field-for-field identical to
/// `stack_model::PermissionPreset` (the resolver maps Declare → compiler
/// type by field-for-field clone). The Declare type exists so the
/// public harvest API doesn't have to expose the L0 `compiler` types
/// to `defineConfig` callers — the same precedent
/// `McpServerDeclare` set with slice 2c.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct PermissionPresetDeclare {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
    #[serde(default)]
    pub additional_directories: Vec<String>,
}
/// Stack-fingerprinted field sets — only consumed by the `drift_tests`
/// module (asserts the Rust harvest matches the generated stub fingerprints).
/// `#[allow(dead_code)]` keeps the lib build green; the constants are part
/// of the agent-config-eval public API surface.
#[allow(dead_code)]
const CONFIG_DEF_FIELDS: &[&str] = &[
    "agents_dir",
    "collections_dir",
    "dependencies",
    "hooks_dir",
    "pipelines_dir",
    "skills_dir",
    "triggers_dir",
];

// ─── Pass-through nouns: field expectations ────────────────────────────────────
// `stack_model::{PipelineDef,TriggerDef,...}` deserialize directly — no
// separate Declare wrapper. These hand-maintained field lists are the
// drift-test anchor: a stack shape change that adds/removes a field
// must update BOTH the compiler type (`stack_model::types::{pipeline,
// trigger,collection}`) and the matching list here, or `drift_tests`
// below fails.
#[allow(dead_code)]
const PIPELINE_DEF_DECLARE_FIELDS: &[&str] = &["auto_merge", "name", "steps"];
#[allow(dead_code)]
const PIPELINE_STEP_DECLARE_FIELDS: &[&str] = &["after", "gate", "id", "step_specs"];
#[allow(dead_code)]
const PIPELINE_GATE_DECLARE_FIELDS: &[&str] = &["forbid_changed", "require_changed"];
#[allow(dead_code)]
const PIPELINE_STEP_SPEC_DECLARE_FIELDS: &[&str] = &["agent", "harness", "id", "prompt"];
#[allow(dead_code)]
const TRIGGER_DEF_DECLARE_FIELDS: &[&str] = &["actions", "id", "source", "trigger"];
#[allow(dead_code)]
const TRIGGER_EVENT_MATCH_DECLARE_FIELDS: &[&str] = &["event"];
#[allow(dead_code)]
const TRIGGER_ACTION_DECLARE_FIELDS: &[&str] = &["type"];
#[allow(dead_code)]
const TRIGGER_SOURCE_WEBHOOK_DECLARE_FIELDS: &[&str] = &["kind", "path", "secret"];
#[allow(dead_code)]
const TRIGGER_SOURCE_FILE_DECLARE_FIELDS: &[&str] = &["glob", "kind"];
#[allow(dead_code)]
const TRIGGER_SOURCE_CRON_DECLARE_FIELDS: &[&str] = &["kind", "schedule"];
#[allow(dead_code)]
const COLLECTION_FIELD_DEF_DECLARE_FIELDS: &[&str] = &[
    "default",
    "filterable",
    "name",
    "required",
    "sortable",
    "type",
    "values",
];
// `defineView` is HAND-SPEC (precedent: `defineRule`, see decision-morning-answers.md §2):
// `ViewDescriptor` is provisional pending sail/deck alignment (decision-views-deck-boundary.md),
// so the fingerprint constant here mirrors the compiler type's field set directly.
// The drift-test arm below cross-checks against `ship_ui::ViewDescriptor`'s
// exported fields, not a stack-derived fingerprint.
#[allow(dead_code)]
const VIEW_DESCRIPTOR_DECLARE_FIELDS: &[&str] = &["component_ref", "prefix", "route"];

#[cfg(test)]
mod drift_tests {
    use super::*;
    use crate::generated_stub as stub;

    /// Both sides sorted, so declaration order in either list is
    /// immaterial — only the field SET must match.
    fn assert_fields_match(label: &str, hand_written: &[&str], generated: &[&str]) {
        let mut hw: Vec<&str> = hand_written.to_vec();
        hw.sort_unstable();
        let mut gend: Vec<&str> = generated.to_vec();
        gend.sort_unstable();
        assert_eq!(
            hw, gend,
            "{label}: the hand-written harvest field list drifted from the stack-generated \
             fingerprint — update the compiler type AND this list together"
        );
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn harvest_fields_match_generated_fingerprints() {
        assert_fields_match(
            "McpServerDef",
            MCP_SERVER_DECLARE_FIELDS,
            stub::MCP_SERVER_DEF_FIELDS,
        );
        assert_fields_match(
            "PipelineDef",
            PIPELINE_DEF_DECLARE_FIELDS,
            stub::PIPELINE_DEF_FIELDS,
        );
        assert_fields_match(
            "PipelineStep",
            PIPELINE_STEP_DECLARE_FIELDS,
            stub::PIPELINE_STEP_FIELDS,
        );
        assert_fields_match(
            "PipelineGate",
            PIPELINE_GATE_DECLARE_FIELDS,
            stub::PIPELINE_GATE_FIELDS,
        );
        assert_fields_match(
            "PipelineStepSpec",
            PIPELINE_STEP_SPEC_DECLARE_FIELDS,
            stub::PIPELINE_STEP_SPEC_FIELDS,
        );
        assert_fields_match(
            "TriggerDef",
            TRIGGER_DEF_DECLARE_FIELDS,
            stub::TRIGGER_DEF_FIELDS,
        );
        assert_fields_match(
            "TriggerEventMatch",
            TRIGGER_EVENT_MATCH_DECLARE_FIELDS,
            stub::TRIGGER_EVENT_MATCH_FIELDS,
        );
        assert_fields_match(
            "TriggerAction",
            TRIGGER_ACTION_DECLARE_FIELDS,
            stub::TRIGGER_ACTION_FIELDS,
        );
        assert_fields_match(
            "TriggerSourceWebhook",
            TRIGGER_SOURCE_WEBHOOK_DECLARE_FIELDS,
            stub::TRIGGER_SOURCE_WEBHOOK_FIELDS,
        );
        assert_fields_match(
            "TriggerSourceFile",
            TRIGGER_SOURCE_FILE_DECLARE_FIELDS,
            stub::TRIGGER_SOURCE_FILE_FIELDS,
        );
        assert_fields_match(
            "TriggerSourceCron",
            TRIGGER_SOURCE_CRON_DECLARE_FIELDS,
            stub::TRIGGER_SOURCE_CRON_FIELDS,
        );
        assert_fields_match(
            "CollectionFieldDef",
            COLLECTION_FIELD_DEF_DECLARE_FIELDS,
            stub::COLLECTION_FIELD_DEF_FIELDS,
        );
        assert_fields_match("ConfigDef", CONFIG_DEF_FIELDS, stub::CONFIG_DEF_FIELDS);
    }

    /// `defineView` is a hand-spec shape (no stack fingerprint — the
    /// `defineRule` precedent applies). The drift check pins the harvest's
    /// field set to `ship_ui::ViewDescriptor`'s serialized fields:
    /// changing one without the other breaks the `defineView` pass-through
    /// harvest in a non-obvious way. The compiler type is the source of
    /// truth; this list mirrors it. A future stack-fingerprinted version
    /// replaces this with `assert_fields_match(..., stub::VIEW_FIELDS)`.
    #[test]
    fn view_descriptor_hand_spec_matches_compiler_type() {
        let serialized = serde_json::to_value(ViewDescriptor::default()).unwrap();
        let fields: std::collections::BTreeSet<&str> = serialized
            .as_object()
            .expect("ViewDescriptor serializes to a JSON object")
            .keys()
            .map(String::as_str)
            .collect();
        let expected: std::collections::BTreeSet<&str> =
            VIEW_DESCRIPTOR_DECLARE_FIELDS.iter().copied().collect();
        assert_eq!(
            fields, expected,
            "ViewDescriptor hand-spec drift: the compiler type's serialized field set \
             must match VIEW_DESCRIPTOR_DECLARE_FIELDS — update the harvest list when \
             ViewDescriptor gains/loses a field"
        );
    }
}
