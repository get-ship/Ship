pub mod agents;
// compile moved to the agent-def crate (Floki carve D2a.2); re-exported so
// `crate::services::compile::*` callers keep resolving (back-compat shim).
pub use agent_def::compile;
pub mod errors;
pub mod mcp;
pub mod rules;
pub mod sessions;
pub mod skills;

#[cfg(any(test, feature = "fixtures"))]
pub fn data_op_examples() -> Vec<(&'static str, serde_json::Value)> {
    let mut out: Vec<(&'static str, serde_json::Value)> = Vec::new();
    out.extend(agents::fixtures::examples());
    out.extend(rules::fixtures::examples());
    out.extend(mcp::fixtures::examples());
    out
}
