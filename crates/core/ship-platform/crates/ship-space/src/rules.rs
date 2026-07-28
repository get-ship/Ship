//! `rules` — runtime policy rows for a space.
//!
//! A *space rule* is a piece of agent guidance authored at the space tier (the
//! runtime scope that follows whichever workspace is active), distinct from
//! per-project rules that ship with a stack via `.ship/rules/*.md`. Every rule
//! is stored as a single `settings_overlay` row in the active space's
//! `space.db`, keyed `rules.<name>` with a JSON body of shape:
//!
//! ```json
//! { "body": "string", "order": number, "enabled": bool, "attribution": "string|null" }
//! ```
//!
//! Reads go through `ship_config::overlay::get_all` — the same per-scope typed
//! reader the `SettingsService` uses for the SPACE scope. Writes travel the
//! canonical path (SettingsService `Set(scope=Space, key, value)` →
//! `overlay::set`); the entity author lives in whatever future surface is
//! created to manage these rows, this module is the read side.
//!
//! Compile-time composition (the consumer side) lives in
//! `apps/helmd/src/main.rs`'s `HarnessService::Compile` handler, which
//! appends each `SpaceRule` to the resolved `agent_schema::Rule` set before the
//! harness adaptors emit context content — so the `space:` provenanced bodies
//! ride along with project rules in the same compile flow.
//!
//! No new tables, projections, protos, or RPCs are introduced — the storage
//! surface is the existing per-space `settings_overlay` partition owned by
//! `ship_config` (settings.proto's SPACE scope).

use std::sync::Arc;

use ship_config::overlay;
use ship_substrate::data::DataService;
use ship_substrate::db;
use ship_substrate::ports::{DataStore, SingleBackendDataStore};

use crate::error::SpaceError;

/// Prefix every rule row's key carries. The full key is `format!("{RULES_KEY_PREFIX}{name}")`;
/// overlay rows that don't share it are not space rules (the partition also
/// holds the per-space SettingsService rows that survive this cutover — the
/// prefix lets the reader filter without disturbing them).
pub const RULES_KEY_PREFIX: &str = "rules.";

/// One row of the space's runtime policy. Mirrors the stored JSON body plus
/// the rule's `name` recovered from the key suffix.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceRule {
    /// The rule's addressable name — the suffix after `rules.` in the
    /// `settings_overlay` key.
    pub name: String,
    /// The body of the rule (the markdown / prose the materializer emits).
    pub body: String,
    /// Display order — ties broken by `name` ascending. `i64` matches the
    /// JSON shape; missing / non-numeric defaults to `0`.
    pub order: i64,
    /// When `false` the row is present but excluded from
    /// [`list_space_rules`] (soft delete + recall).
    pub enabled: bool,
    /// Who wrote this row. `None` when unset (legacy rows) or when the writer
    /// passes `null` to mean "anon / system".
    pub attribution: Option<String>,
}

impl SpaceRule {
    /// Project this rule into the `agent_schema::Rule` shape the harness
    /// adaptors consume during compile. The body travels as-is (no provenance
    /// line in `content`); the `space:<name>` prefix lives in `file_name` so
    /// any surface that surfaces `file_name` (Codex per-rule files, future
    /// rule indexers) can show the provenance, and the harness adaptor's
    /// context materializer emits the body verbatim into CLAUDE.md alongside
    /// any project rule bodies.
    pub fn into_stack_rule(self) -> agent_schema::Rule {
        agent_schema::Rule {
            file_name: format!("space:{}", self.name),
            content: self.body,
            always_apply: true,
            globs: Vec::new(),
            description: None,
        }
    }
}

/// The stored JSON shape persisted at every `rules.<name>` key. Kept private —
/// callers build & read [`SpaceRule`] through [`list_space_rules`], so the
/// shape stays the implementation detail of *this* module.
#[derive(Debug, serde::Deserialize)]
struct StoredRule {
    body: Option<String>,
    order: Option<serde_json::Value>,
    enabled: Option<bool>,
    attribution: Option<serde_json::Value>,
}

impl StoredRule {
    fn into_space_rule(self, name: String) -> SpaceRule {
        SpaceRule {
            name,
            body: self.body.unwrap_or_default(),
            order: self.order.and_then(coerce_i64).unwrap_or(0),
            enabled: self.enabled.unwrap_or(true),
            attribution: self.attribution.and_then(string_or_null),
        }
    }
}

/// Coerce a JSON value to `i64` the same way `ship_config::settings::coerce_u64`
/// does for the daemon settings tier — accept a JSON number, or a stringified
/// number (settings are persisted as JSON; older writers may have stringified).
fn coerce_i64(v: serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// `Some(s)` when the value is a JSON string, `None` when it is JSON `null`
/// (the spec's wire shape for "no attribution"), `None` for any other shape
/// (forward-compatible: a malformed row drops attribution rather than
/// crashing the read).
fn string_or_null(v: serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Null => None,
        _ => None,
    }
}

/// Open the `<ship_dir>/spaces/<space_id>/space.db` partition as a
/// `SingleBackendDataStore`. The `partition` label is the space id itself —
/// the partition layer uses the db file as the scope boundary, so the row
/// only needs the setting key as its id.
///
/// Lazily provisions the underlying sqlite db; a `space_id` that has never
/// opened resolves to an empty store (no settings_overlay table yet) and the
/// caller reads an empty list — the same graceful-empty behaviour every
/// overlay consumer exhibits.
fn open_space_store(
    ship_dir: &std::path::Path,
    space_id: &str,
) -> Result<Arc<dyn DataStore>, SpaceError> {
    let db_path = db::space_db_path(ship_dir, space_id);
    let svc = DataService::open_at(&db_path).map_err(|e| SpaceError::Internal(e.to_string()))?;
    Ok(Arc::new(SingleBackendDataStore::new(svc, space_id)))
}

/// List the rules stored for `space_id`, ordered by ascending `order` then
/// `name`. Rows whose `enabled` field is `false` are excluded.
///
/// Storage: the per-space `settings_overlay` partition — the same read-only
/// path the SPACE-scope `SettingsService::list` takes for the runtime
/// settings surface. The reader filters on the `rules.` key prefix and
/// ignores every other overlay row (the daemon ships other settings keys
/// here; this module owns the `rules.*` namespace, nothing else).
///
/// Errors:
/// - `SpaceError::Internal` when the underlying db cannot be opened or the
///   overlay reader fails. A never-opened space resolves to `Ok(vec![])`.
pub async fn list_space_rules(space_id: &str) -> Result<Vec<SpaceRule>, SpaceError> {
    let ship_dir = ship_substrate::project::get_global_dir()
        .map_err(|e| SpaceError::Internal(format!("resolve ship data root: {e}")))?;
    let store = open_space_store(&ship_dir, space_id)?;
    let raw = overlay::get_all(store.as_ref())
        .await
        .map_err(|e| SpaceError::Internal(e.to_string()))?;

    let mut rules: Vec<SpaceRule> = raw
        .into_iter()
        .filter_map(|(key, value)| {
            let name = key.strip_prefix(RULES_KEY_PREFIX)?.to_string();
            // Skip empty rule names — the prefix itself wouldn't match
            // here, but a `rules.` (empty suffix) row is a corruption
            // we drop instead of crashing the read.
            if name.is_empty() {
                return None;
            }
            let parsed: StoredRule = serde_json::from_value(value).ok()?;
            let rule = parsed.into_space_rule(name);
            // enabled: false is a soft delete + recall — exclude per
            // contract; preserved in storage so callers can re-enable.
            rule.enabled.then_some(rule)
        })
        .collect();

    // Order: ascending `order`, ties broken by `name` ascending (string
    // compare) — a stable, content-respecting total order.
    rules.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.name.cmp(&b.name)));
    Ok(rules)
}
