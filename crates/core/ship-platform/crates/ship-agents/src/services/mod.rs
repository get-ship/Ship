pub mod agents;
pub mod compile;
pub mod mcp;
pub mod permissions;
pub mod rules;
pub mod sessions;
pub mod skills;

#[cfg(any(test, feature = "fixtures"))]
pub fn data_op_examples() -> Vec<(&'static str, serde_json::Value)> {
    let mut out: Vec<(&'static str, serde_json::Value)> = Vec::new();
    out.extend(agents::fixtures::examples());
    out.extend(rules::fixtures::examples());
    out.extend(mcp::fixtures::examples());
    out.extend(permissions::fixtures::examples());
    out
}
