//! Managed MCP state — server ids and last mode per provider.
//!
//! Stored as a single JSON row per provider in `kv_state` under the
//! `mcp.managed` namespace. Each provider's value is `{ "server_ids":
//! [...], "last_mode": "..." }`. Two scalar fields per provider don't
//! warrant a dedicated table.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::kv;

const NAMESPACE: &str = "mcp.managed";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ManagedState {
    #[serde(default)]
    server_ids: Vec<String>,
    #[serde(default)]
    last_mode: Option<String>,
}

/// Returns `(server_ids, last_mode)` for the given provider, or empty defaults.
pub fn get_managed_state_db(provider: &str) -> Result<(Vec<String>, Option<String>)> {
    let Some(value) = kv::get(NAMESPACE, provider)? else {
        return Ok((Vec::new(), None));
    };
    let state: ManagedState = serde_json::from_value(value).unwrap_or_default();
    Ok((state.server_ids, state.last_mode))
}

/// Persist the managed server ids and last mode for the given provider.
pub fn set_managed_state_db(
    provider: &str,
    ids: &[String],
    last_mode: Option<&str>,
) -> Result<()> {
    let state = ManagedState {
        server_ids: ids.to_vec(),
        last_mode: last_mode.map(str::to_string),
    };
    let value = serde_json::to_value(&state)?;
    kv::set(NAMESPACE, provider, &value)
}
