//! Harvest every declare-form kind emitted by a `ship.config.ts` eval — not just
//! `defineAgent`. [`generated_stub::SDK_STUB`] tags each declare-form's
//! emitted value with a `__kind` discriminant (`{ __kind, value }`); this
//! module routes each tagged value to its noun's harvest shape.
//!
//! Two harvest strategies, per noun:
//!
//! - **Pass-through** (pipelines, triggers): the stack authoring shape and
//!   `compiler`'s type are identical, so the emitted value deserializes
//!   directly into `compiler::PipelineDef` / `compiler::TriggerDef`.
//! - **Mapped** (mcp servers, skills): `compiler`'s type predates the
//!   stack shape and carries fields the authoring shape doesn't (a
//!   display `name` distinct from `id`, Codex/Cursor per-server knobs,
//!   etc.) — a private `*Declare` struct mirrors the stack shape exactly
//!   and a `into_*` fn maps it onto the compiler type, filling the gap
//!   with the same defaults a fresh `.jsonc`/`SKILL.md` entry would get.
//! - **Rules** are neither: per `decision-morning-answers.md` §2, the
//!   rules declare-form is a thinnest-possible hand-specified shape (id +
//!   content + apply scope), deliberately NOT stack's richer `RuleDef` —
//!   see [`RuleDeclare`].
//!
//! Every mapped/pass-through noun's hand-written field set is checked
//! against [`generated_stub`]'s stack-derived fingerprints by
//! `drift_tests` below — the Rust-side half of the drift gate
//! `gen_eval_stub`'s `--check` opens at the TS-shape layer.

use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use compiler::{
    AgentProfile, CollectionDef, McpServerConfig, McpServerType, PipelineDef, Rule, Skill,
    SkillSource, TriggerDef,
};
use compiler::types::ViewDescriptor;
use serde::Deserialize;
use serde_json::Value;

/// Every declare-form's harvested output from one `ship.config.ts` eval, merged
/// in call order. [`crate::resolve_agents_with_globals`] is a thin filter
/// over `.agents`; `ts_resolver::resolve_ts_files` merges these again
/// across multiple modules.
#[derive(Debug, Clone, Default)]
pub struct Declarations {
    pub agents: Vec<AgentProfile>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<Skill>,
    pub rules: Vec<Rule>,
    pub pipelines: Vec<PipelineDef>,
    pub triggers: Vec<TriggerDef>,
    pub collections: Vec<CollectionDef>,
    /// Views harvested from `defineView` calls. The composition law
    /// (`compiler::compose::compose_space`) merges these by namespace-prefix
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
    /// fan-out and rolls the per-module values into
    /// `ExtensionLibrary::skill_paths` post-eval.
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
pub fn harvest(out: &mut Declarations, index: usize, raw: Value) -> Result<()> {
    let envelope: Envelope = serde_json::from_value(raw)
        .with_context(|| format!("declare-form call #{index} did not emit a tagged declaration"))?;
    match envelope.__kind.as_str() {
        "agent" => {
            let profile: AgentProfile = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineAgent call #{index} did not match AgentProfile"))?;
            out.agents.push(profile);
        }
        "mcp_server" => {
            let declared: McpServerDeclare = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineMcpServer call #{index} did not match McpServerDef"))?;
            out.mcp_servers.push(declared.into_mcp_server_config());
        }
        "skill" => {
            let declared: SkillDeclare = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineSkill call #{index} did not match SkillDef"))?;
            out.skills.push(declared.into_skill());
        }
        "rule" => {
            let declared: RuleDeclare = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineRule call #{index} did not match the rule shape"))?;
            out.rules.push(declared.into_rule(index)?);
        }
        "pipeline" => {
            let def: PipelineDef = serde_json::from_value(envelope.value)
                .with_context(|| format!("definePipeline call #{index} did not match PipelineDef"))?;
            out.pipelines.push(def);
        }
        "trigger" => {
            let def: TriggerDef = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineTrigger call #{index} did not match TriggerDef"))?;
            out.triggers.push(def);
        }
        "collection" => {
            let def: CollectionDef = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineCollection call #{index} did not match the collection shape"))?;
            out.collections.push(def);
        }
        "view" => {
            // The `defineView` value is the compiler's `ViewDescriptor` shape
            // (route/prefix/component_ref) — a pass-through, mirroring
            // `definePipeline` / `defineCollection`. The composition law
            // (`compiler::compose::compose_space`) merges the resulting
            // `out.views` by namespace-prefix merge.
            let def: ViewDescriptor = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineView call #{index} did not match ViewDescriptor"))?;
            out.views.push(def);
        }
        "config" => {
            // The `defineConfig` value carries the per-extension-project
            // `skills_dir` (single dir name, no trailing slash, relative
            // to the dir containing the `ship.config.ts` declaring it).
            // Hand-spec'd (no `@ship/stack` counterpart yet) — the same
            // posture as `defineRule` / `defineView`. The resolver's
            // `merge_ts_declarations` walks `out.config` and threads
            // the value into `ExtensionLibrary::skill_paths` (scalar
            // override — the last `defineConfig.skills_dir` wins).
            let declared: ConfigDeclare = serde_json::from_value(envelope.value)
                .with_context(|| format!("defineConfig call #{index} did not match the config shape"))?;
            out.config.push(declared);
        }
        other => bail!("declare-form call #{index}: unrecognized kind `{other}`"),
    }
    Ok(())
}

// ─── MCP servers ──────────────────────────────────────────────────────────────

/// The `McpServerDef` authoring shape (`@ship/stack/mcp-servers.ts`).
#[derive(Debug, Deserialize)]
struct McpServerDeclare {
    id: String,
    server_type: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    url: Option<String>,
    scope: String,
}

const MCP_SERVER_DECLARE_FIELDS: &[&str] =
    &["args", "command", "env", "id", "scope", "server_type", "url"];

impl McpServerDeclare {
    /// Map onto `compiler::McpServerConfig`. The authoring shape has no
    /// separate display name (only `id`) — `name` doubles as `id`, same
    /// as `defineAgent`'s `id`/`name` collapse when a config omits one.
    /// `disabled`/`timeout_secs`/the Codex+Cursor per-server fields have
    /// no TS-authored equivalent yet; they default off, same as a fresh
    /// `mcp.jsonc` entry would.
    fn into_mcp_server_config(self) -> McpServerConfig {
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

// ─── Skills ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SkillGrantDeclare {
    tool: String,
    #[serde(default)]
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SkillRequirementDeclare {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SkillAssetRefDeclare {
    #[allow(dead_code)]
    kind: String,
    #[allow(dead_code)]
    path: String,
}

const SKILL_ASSET_REF_DECLARE_FIELDS: &[&str] = &["kind", "path"];

#[derive(Debug, Deserialize)]
struct SkillFrontmatterDeclare {
    name: String,
    #[serde(rename = "stable-id")]
    stable_id: Option<String>,
    description: String,
    license: Option<String>,
    compatibility: Option<Value>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<Vec<String>>,
    artifacts: Option<Vec<String>>,
    grants: Option<Vec<SkillGrantDeclare>>,
    requires: Option<Vec<SkillRequirementDeclare>>,
    metadata: Option<HashMap<String, String>>,
}

const SKILL_FRONTMATTER_DECLARE_FIELDS: &[&str] = &[
    "allowed-tools",
    "artifacts",
    "compatibility",
    "description",
    "grants",
    "license",
    "metadata",
    "name",
    "requires",
    "stable-id",
];

/// The `SkillDef` authoring shape (`@ship/stack/skill-def.ts`).
#[derive(Debug, Deserialize)]
struct SkillDeclare {
    frontmatter: SkillFrontmatterDeclare,
    body: String,
    #[serde(default)]
    #[allow(dead_code)]
    assets: Vec<SkillAssetRefDeclare>,
}

const SKILL_DEF_DECLARE_FIELDS: &[&str] = &["assets", "body", "frontmatter"];

impl SkillDeclare {
    /// Map onto `compiler::Skill`.
    ///
    /// KNOWN GAP (flagged, not fixed here): `compiler::Skill` has no
    /// asset-bundle field yet, so `assets` is harvested (proves the wire
    /// shape round-trips) but dropped at this boundary — a future wave
    /// must extend `Skill` before script/reference bundles reach the
    /// compile pipeline. `grants`/`requires` similarly collapse from
    /// stack's `{tool/id, scope?/version?}` objects to bare id strings —
    /// `compiler::Skill`'s grant model has no per-grant scope yet either.
    fn into_skill(self) -> Skill {
        let fm = self.frontmatter;
        let id = fm.stable_id.clone().unwrap_or_else(|| slugify(&fm.name));
        Skill {
            id,
            name: fm.name,
            stable_id: fm.stable_id,
            description: Some(fm.description),
            license: fm.license,
            compatibility: fm.compatibility.map(|v| v.to_string()),
            allowed_tools: fm.allowed_tools.unwrap_or_default(),
            metadata: fm.metadata.unwrap_or_default(),
            content: self.body,
            source: SkillSource::Custom,
            vars: HashMap::new(),
            artifacts: fm.artifacts.unwrap_or_default(),
            grants: fm.grants.unwrap_or_default().into_iter().map(|g| g.tool).collect(),
            requires: fm.requires.unwrap_or_default().into_iter().map(|r| r.id).collect(),
        }
    }
}

/// Mirrors `compiler::skill_parser`'s directory-name normalization: a
/// skill declared without `stable-id` gets one derived from its display
/// name the same way a `SKILL.md`'s `name:` frontmatter would.
fn slugify(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
// ─── Per-extension-project config (defineConfig) ──────────────────────────
//
// The `defineConfig({ skills_dir: "..." })` value (a per-extension-project
// authoring shape, declared as `export default defineConfig({...})` from
// each extension's `ship.config.ts` for ecosystem conformance). The
// `skills_dir` value is a SINGLE directory name (no trailing slash)
// where on-disk skills live — the resolver walks
// `<extension_dir>/<skills_dir>/<id>/SKILL.md` and produces `Skill`
// records (the `defineSkill` declare-form + body-materialization is a
// follow-up job, out of scope here). Hand-spec'd (no `@ship/stack`
// counterpart yet) — the same posture as `defineRule` / `defineView`.
// The Rust harvest pushes one entry per `defineConfig` call into
// `Declarations::config`; the resolver's `merge_ts_declarations`
// walks that vec and threads the per-module `skills_dir` into
// `ExtensionLibrary::skill_paths` post-eval.
//
// The default (when no `defineConfig` is declared) is the unified
// `DEFAULT_SPACE_SKILL_PATH` ("skills/") which the compiler leaf
// already populates into the descriptor. Per-extension-project
// `defineConfig` overrides that.

/// The `defineConfig` value — a per-extension-project authoring shape.
/// Top-level keys: `skills_dir` (the only key supported today; a single
/// dir name, no trailing slash). An empty `defineConfig({})` is valid
/// (the resolver sees an empty `ConfigDeclare` and falls through to
/// the unified default).
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConfigDeclare {
    #[serde(default)]
    pub skills_dir: Option<String>,
}

const CONFIG_DEF_FIELDS: &[&str] = &["skills_dir"];

// ─── Rules ──────────────────────────────────────────────────────────────────

/// The rules declare-form's shape — deliberately NOT `@ship/stack`'s
/// `RuleDef` (frontmatter + body). Per `decision-morning-answers.md` §2,
/// the ruling collapses rules to the thinnest form: id, content, and a
/// cascade placement ("apply scope") — no markdown indirection, no rich
/// type. Not derived from stack, hence excluded from `gen_eval_stub`'s
/// fingerprinting (see that module's doc comment) and from
/// `drift_tests` below.
#[derive(Debug, Deserialize)]
struct RuleDeclare {
    id: String,
    content: String,
    #[serde(rename = "applyScope")]
    apply_scope: RuleApplyScopeDeclare,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RuleApplyScopeDeclare {
    Tag(String),
    Scoped { globs: Vec<String> },
}

impl RuleDeclare {
    fn into_rule(self, index: usize) -> Result<Rule> {
        let (always_apply, globs) = match self.apply_scope {
            RuleApplyScopeDeclare::Tag(tag) if tag == "always" => (true, Vec::new()),
            RuleApplyScopeDeclare::Tag(other) => bail!(
                "defineRule call #{index} (`{}`): applyScope string must be \"always\", got \"{other}\"",
                self.id
            ),
            RuleApplyScopeDeclare::Scoped { globs } => (false, globs),
        };
        Ok(Rule {
            file_name: self.id,
            content: self.content,
            always_apply,
            globs,
            description: None,
        })
    }
}

// ─── Pass-through nouns: field expectations ────────────────────────────────────
// `compiler::{PipelineDef,TriggerDef,...}` deserialize directly — no
// separate Declare wrapper. These hand-maintained field lists are the
// drift-test anchor: a stack shape change that adds/removes a field
// must update BOTH the compiler type (`compiler::types::{pipeline,
// trigger,collection}`) and the matching list here, or `drift_tests`
// below fails.

const PIPELINE_DEF_DECLARE_FIELDS: &[&str] = &["auto_merge", "name", "steps"];
const PIPELINE_STEP_DECLARE_FIELDS: &[&str] = &["after", "gate", "id", "step_specs"];
const PIPELINE_GATE_DECLARE_FIELDS: &[&str] = &["forbid_changed", "require_changed"];
const PIPELINE_STEP_SPEC_DECLARE_FIELDS: &[&str] = &["agent", "harness", "id", "prompt"];
const TRIGGER_DEF_DECLARE_FIELDS: &[&str] = &["actions", "id", "source", "trigger"];
const TRIGGER_EVENT_MATCH_DECLARE_FIELDS: &[&str] = &["event"];
const TRIGGER_ACTION_DECLARE_FIELDS: &[&str] = &["type"];
const TRIGGER_SOURCE_WEBHOOK_DECLARE_FIELDS: &[&str] = &["kind", "path", "secret"];
const TRIGGER_SOURCE_FILE_DECLARE_FIELDS: &[&str] = &["glob", "kind"];
const TRIGGER_SOURCE_CRON_DECLARE_FIELDS: &[&str] = &["kind", "schedule"];
const COLLECTION_FIELD_DEF_DECLARE_FIELDS: &[&str] =
    &["default", "filterable", "name", "required", "sortable", "type", "values"];
// `defineView` is HAND-SPEC (precedent: `defineRule`, see decision-morning-answers.md §2):
// `ViewDescriptor` is provisional pending sail/deck alignment (decision-views-deck-boundary.md),
// so the fingerprint constant here mirrors the compiler type's field set directly.
// The drift-test arm below cross-checks against `compiler::types::ViewDescriptor`'s
// exported fields, not a stack-derived fingerprint.
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

    #[test]
    fn harvest_fields_match_generated_fingerprints() {
        assert_fields_match("McpServerDef", MCP_SERVER_DECLARE_FIELDS, stub::MCP_SERVER_DEF_FIELDS);
        assert_fields_match("SkillDef", SKILL_DEF_DECLARE_FIELDS, stub::SKILL_DEF_FIELDS);
        assert_fields_match(
            "SkillFrontmatter",
            SKILL_FRONTMATTER_DECLARE_FIELDS,
            stub::SKILL_FRONTMATTER_FIELDS,
        );
        assert_fields_match(
            "SkillAssetRef",
            SKILL_ASSET_REF_DECLARE_FIELDS,
            stub::SKILL_ASSET_REF_FIELDS,
        );
        assert_fields_match("PipelineDef", PIPELINE_DEF_DECLARE_FIELDS, stub::PIPELINE_DEF_FIELDS);
        assert_fields_match("PipelineStep", PIPELINE_STEP_DECLARE_FIELDS, stub::PIPELINE_STEP_FIELDS);
        assert_fields_match("PipelineGate", PIPELINE_GATE_DECLARE_FIELDS, stub::PIPELINE_GATE_FIELDS);
        assert_fields_match(
            "PipelineStepSpec",
            PIPELINE_STEP_SPEC_DECLARE_FIELDS,
            stub::PIPELINE_STEP_SPEC_FIELDS,
        );
        assert_fields_match("TriggerDef", TRIGGER_DEF_DECLARE_FIELDS, stub::TRIGGER_DEF_FIELDS);
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
        assert_fields_match(
            "ConfigDef",
            CONFIG_DEF_FIELDS,
            stub::CONFIG_DEF_FIELDS,
        );
    }

    /// `defineView` is a hand-spec shape (no stack fingerprint — the
    /// `defineRule` precedent applies). The drift check pins the harvest's
    /// field set to `compiler::types::ViewDescriptor`'s serialized fields:
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
