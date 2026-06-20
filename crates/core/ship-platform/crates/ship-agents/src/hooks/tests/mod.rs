/// Integration tests for the hooks module.
///
/// Sub-domains:
/// - `tool_gate`     — gate integration: contextual deny/allow flows not covered by
///                     the unit tests in `tool_gate_tests.rs`.
///
/// Test naming follows `hooks_<subdomain>_<scenario>` for coverage-tool matching.
/// Each test is hermetic — no shared state, no tempdir reuse across tests.
#[cfg(test)]
mod integration_tool_gate;
