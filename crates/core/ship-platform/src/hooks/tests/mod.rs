/// Integration tests for the hooks module.
///
/// Sub-domains:
/// - `hook_events`   — handler pipeline: envelope shape, actor/workspace context,
///                     session+tool lifecycle sequences.
/// - `tool_gate`     — gate integration: contextual deny/allow flows not covered by
///                     the unit tests in `tool_gate_tests.rs`.
/// - `soft_delete`   — filesystem side-effect observation via `maybe_trash_before_tool`.
///
/// Test naming follows `hooks_<subdomain>_<scenario>` for coverage-tool matching.
/// Each test is hermetic — no shared state, no tempdir reuse across tests.
#[cfg(test)]
mod integration_hook_events;
#[cfg(test)]
mod integration_tool_gate;
#[cfg(test)]
mod integration_soft_delete;
