//! Per-harness data-dir mining.
//!
//! Every supported harness writes substantial state to disk under
//! `~/.<harness>/` — session transcripts, usage logs, settings,
//! conversation history. Ship is effectively a GUI on top of these
//! harnesses; reading the data they already maintain is strictly
//! richer than reconstructing it via hooks:
//!
//!   * historical coverage (sessions started before Ship was installed)
//!   * authoritative numbers (the harness records its own usage)
//!   * format completeness (full prompts + outputs, not 400-char
//!     previews)
//!
//! ## Sources today
//!
//! | Harness  | Location                              | What lives there |
//! |----------|---------------------------------------|------------------|
//! | claude   | `~/.claude/projects/<encoded_path>/<session>.jsonl` | full transcripts, usage blocks per turn |
//! | claude   | `~/.claude/statsig/`                  | telemetry / cost (format proprietary) |
//! | claude   | `~/.claude/todos/`                    | task state |
//! | codex    | `~/.codex/sessions/`                  | similar transcripts |
//! | codex    | `~/.codex/history.jsonl`              | running command history |
//! | cursor   | mostly cloud-stored                   | hard target |
//! | opencode | `~/.opencode/` (TBD)                  | plugin model — different shape |
//!
//! ## Architecture
//!
//! Each `<harness>` module owns:
//!   1. A read-only parser for that harness's on-disk format.
//!   2. A `scan(...) -> Vec<MinedEvent>` one-shot importer (catches
//!      up history into analytics_sessions / analytics_tool_uses).
//!   3. A `watch(...)` future that fires on file changes (inotify /
//!      polling fallback for WSL) and emits the same events
//!      continuously.
//!
//! ## What gets emitted
//!
//! Mining produces synthetic `hook.*` events (same shape as
//! live-stream hooks) so the existing AnalyticsProjection apply
//! path handles them uniformly. Mined events carry a source flag
//! in their payload (`source: "mined"`) so analytics consumers can
//! distinguish live signal from historical fill.
//!
//! ## Privacy
//!
//! Default: local-only. Mining runs against the local user's
//! `~/.<harness>/` and writes to the local platform.db. No cloud
//! sync without explicit user opt-in. This matches the launch-arc
//! "Cloud sync as paid feature" architectural decision.

pub mod claude;

#[cfg(test)]
mod tests;
