//! Settings commands — runtime settings writes via the platform config layer
//! (`config::set_setting_and_emit`). Reads are settings.list/get.
//!
//! Default C1 today: settings are workspace/user prefs (defaults.harness, theme,
//! …). NOTE(#82): some keys are security-relevant (the future ask-policy,
//! dispatch defaults). When that mechanism lands, those keys must be gated
//! per-key to C2 — a blanket C1 here trusts that no security-load-bearing key
//! exists at C1 yet. The per-key gate is #82's, not this module's.

use serde_json::Value;

use super::{req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "settings.set" => {
            let key = req_str(args, "key")?;
            let raw = req_str(args, "value_json")?;
            let value: Value = serde_json::from_str(&raw).map_err(|e| {
                CommandError::bad_args(format!("`value_json` is not valid JSON: {e}"))
            })?;
            ship_platform::config::set_setting_and_emit(ctx.state.kernel(), &key, value.clone())
                .await
                .map_err(|e| CommandError::service(e.to_string()))?;
            // Echo the typed SettingValue shape (value re-serialized as a string).
            Ok(Some(serde_json::json!({ "key": key, "value_json": value.to_string() })))
        }
        _ => Ok(None),
    }
}
