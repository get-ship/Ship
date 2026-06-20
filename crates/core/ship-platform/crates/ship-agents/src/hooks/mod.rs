//! Hook ingress — CLI/HTTP layer that emits `hook.*` and `skill.*` events
//! for harness hook input. The workbench `handle_session_start` /
//! `handle_session_end` helpers (which emitted `session.started` /
//! `session.ended` workbench events) were retired with the workbench
//! family (job `workbench-teardown`). The `hook.*` family has been
//! replaced by canonical `op.*` events (op.tool_use_started /
//! op.tool_use_completed / op.tool_use_denied) in #25b; hook.* emission from the HTTP layer is retired.

pub mod tool_gate;
pub mod http_ingress;

#[cfg(test)]
mod tests;
