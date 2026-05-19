//! Hook lifecycle event payloads.
//!
//! Emitted by `apps/shipd/src/routes/hooks_api.rs` when a harness's
//! hook configuration fires `ship hook <event>` against the daemon.
//! These typed payloads replace the free-form `json!({...})` shape
//! used in the earlier session and unlock the AnalyticsProjection
//! downstream (per-session duration, per-tool counts, success rates).
//!
//! Backward compatibility: each payload accepts a structured
//! `input` / `response` (`serde_json::Value`) alongside the legacy
//! truncated `input_preview` / `response_preview` strings. Old
//! consumers that read the preview keep working; new consumers
//! (analytics projection, dashboards) prefer the structured fields.
//!
//! Naming note: this module sits next to `pipeline.rs`, `process.rs`,
//! `executor.rs`. The mod-level `event_types` constants get
//! re-exported under `events::types::event_types` for callers that
//! already import from there.

use serde::{Deserialize, Serialize};
use specta::Type;

pub mod event_types {
    pub const HOOK_SESSION_STARTED: &str = "hook.session_started";
    pub const HOOK_SESSION_ENDED: &str = "hook.session_ended";
    pub const HOOK_TOOL_USE_STARTED: &str = "hook.tool_use_started";
    pub const HOOK_TOOL_USE_COMPLETED: &str = "hook.tool_use_completed";
    /// Per-turn token usage. Synthesized by the supervisor from the
    /// harness's structured output stream (claude stream-json,
    /// opencode --format json, codex --json). Source of truth for
    /// token counts. Cost is derived on read.
    pub const HOOK_TURN_OBSERVED: &str = "hook.turn_observed";
    /// A tool call was denied before execution. Emitted today by the
    /// Ship-integrity gate in `ship hook before-tool` when its
    /// fail-closed checks reject a tool call; future operator-approval
    /// flows will emit the same event with denied_by="operator".
    /// Distinct from hook.tool_use_completed{status:error} because the
    /// tool never ran — there is no started/completed pair to roll up.
    pub const HOOK_PERMISSION_DENIED: &str = "hook.permission_denied";
}

/// Fired when a harness reports the start of an interactive session.
/// Drives session-level analytics rollups (per-session duration,
/// per-model token counts, cost).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookSessionStartedPayload {
    /// Ship-side actor id this hook belongs to (typically the
    /// dispatched slug or the supervisor-generated ULID).
    pub actor_id: String,
    /// Harness-native session id (claude's session UUID, codex's
    /// thread id, etc.) when the hook payload carries it. Optional
    /// because not every harness surfaces it on session-start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Harness identifier — "claude", "codex", "cursor", "gemini",
    /// "opencode". Populated from `ShipHookEvent.harness` when set;
    /// `None` for legacy emitters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Model identifier the harness opened the session with. None
    /// when the hook payload doesn't expose it (most harnesses don't
    /// pass model into hook stdin — capture it via the adapter's
    /// stream parser in a follow-up).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// ISO-8601 timestamp of the start observation.
    pub started_at: String,
}

/// Fired when a harness reports the end of a session OR when Ship
/// detects session teardown (cancel / executor.session_destroyed /
/// process exit). Pair with HookSessionStartedPayload for duration
/// rollups.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookSessionEndedPayload {
    pub actor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// ISO-8601 timestamp of the end observation.
    pub ended_at: String,
    /// Operator-facing reason: "operator-cancelled", "stop-hook",
    /// "process-exit", "executor-session-destroyed", "error", or
    /// harness-specific. None when not classified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Tool-use status carried on the completed payload. String form
/// keeps the wire stable across serde versions / language clients.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Ok,
    Error,
    /// Harness didn't expose success/failure for this tool call.
    Unknown,
}

/// Fired before a tool invocation. Carries the structured arguments
/// so analytics can count "all bash commands" or "all file edits to
/// X" — the legacy preview string couldn't support that.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookToolUseStartedPayload {
    pub actor_id: String,
    /// Tool name as the harness reports it. Conventional set:
    /// "Bash", "Read", "Write", "Edit", "Grep", "Glob", "Task",
    /// "WebFetch", and any user-defined MCP tool name.
    pub tool: String,
    /// Structured tool input. Shape is tool-specific (Bash → {command,
    /// description?}; Edit → {file_path, old_string, new_string}). May
    /// be omitted when the harness doesn't surface inputs (rare).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Truncated string preview (≤400 chars) — kept for the dashboard's
    /// existing renderer that expects a string. New consumers should
    /// use `input` for queryability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_preview: Option<String>,
    pub started_at: String,
}

/// Fired after a tool invocation completes (success or failure).
/// Pair with HookToolUseStartedPayload via (actor_id, tool, started_at)
/// for duration computation in the AnalyticsProjection.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookToolUseCompletedPayload {
    pub actor_id: String,
    pub tool: String,
    pub status: ToolStatus,
    /// Error message when status = Error. None on Ok.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Structured tool response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
    /// Truncated string preview (≤400 chars). Legacy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_preview: Option<String>,
    pub ended_at: String,
}

/// Per-turn token usage observation. Synthesized by the supervisor
/// when the harness's structured stream reports a turn boundary with
/// usage info. Universal across harnesses — tokens are provider-emitted,
/// don't go stale, and don't conflate with subscription-vs-API-key
/// pricing concerns (cost is computed on read, never stored).
///
/// All counts are absolute for THIS turn (not cumulative). Zero is a
/// valid value; null/missing means the harness didn't surface that
/// category (e.g. claude pre-cache-API doesn't have cache_creation).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookTurnObservedPayload {
    pub actor_id: String,
    /// Harness-native session id when available, else None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Ordinal of this turn within the session, when the harness
    /// surfaces it. Optional; analytics_turns can still order by
    /// observed_at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_idx: Option<u32>,
    /// Model used for THIS turn — may differ from the session's
    /// initial model on fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
    #[serde(default)]
    pub cache_creation_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
    /// "end_turn" | "max_tokens" | "tool_use" | "stop" | harness-specific.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Cost in USD as reported by the harness (claude's
    /// total_cost_usd in result events). Source of truth for "what
    /// the harness thinks this turn cost" — captured verbatim, never
    /// computed by Ship. None when the harness didn't surface it.
    /// Subscription users may see API-equivalent values that don't
    /// reflect their plan; the dashboard should label this clearly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_reported_cost_usd: Option<f64>,
    pub observed_at: String,
}

/// A tool call was rejected before invocation. Today emitted from
/// `ship hook before-tool` when the fail-closed integrity gate
/// (`cli-framework/src/hooks/tool_gate.rs`) returns `Decision::Deny`;
/// the deny response is sent back to the harness so the tool never
/// runs. Captured separately from hook.tool_use_* because the call
/// never produced a started/completed pair.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HookPermissionDeniedPayload {
    pub actor_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Tool name the harness was about to invoke.
    pub tool: String,
    /// Structured tool input the gate inspected. Useful for forensics
    /// ("which Bash command did we block?"). Optional in case the
    /// harness didn't surface input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Truncated preview (≤400 chars) for legacy renderers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_preview: Option<String>,
    /// Operator-readable reason from the gate. Carries the exact rule
    /// message ("ship hook: blocked write inside .git/ …").
    pub reason: String,
    /// Origin of the denial. Today always "ship-gate"; later
    /// "operator" (UI approval flow) or "harness" (provider-side
    /// permission check) once those wire up.
    pub denied_by: String,
    pub denied_at: String,
}
