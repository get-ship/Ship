//! `ship config` over shipd's `SettingsService` (settings.proto). The CLI is a
//! pure client: shipd must be running or these calls error. Scope is inferred
//! from the key — portable keys (identity/defaults/cloud) are account-scoped;
//! everything else is machine-local (runtime).

use anyhow::{anyhow, Context, Result};
use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{
    GetSettingReq, ListSettingsReq, SetSettingReq, SettingScope, SettingsServiceClient,
};

fn scope_of(key: &str) -> SettingScope {
    if key.starts_with("identity.") || key.starts_with("defaults.") || key.starts_with("cloud.") {
        SettingScope::SETTING_SCOPE_ACCOUNT
    } else {
        SettingScope::SETTING_SCOPE_RUNTIME
    }
}

fn connect() -> Result<(SettingsServiceClient<HttpClient>, tokio::runtime::Runtime)> {
    let (port, token) = crate::daemon::ensure_daemon()
        .map_err(|e| anyhow!("shipd not running: {e}\nTry: ship daemon start"))?;
    let uri: http::Uri =
        format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
    let config = ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = SettingsServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    Ok((client, rt))
}

/// JSON-serialized value bytes → a plain display string (unwrap JSON strings).
fn display_value(bytes: &[u8]) -> String {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(v) => v.to_string(),
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

pub fn get(key: &str) -> Result<Option<String>> {
    let (client, rt) = connect()?;
    let req = GetSettingReq {
        key: key.to_string(),
        scope: scope_of(key).into(),
        __buffa_unknown_fields: Default::default(),
    };
    match rt.block_on(client.get(req)) {
        Ok(resp) => Ok(Some(display_value(&resp.view().value_json))),
        Err(_) => Ok(None),
    }
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let (client, rt) = connect()?;
    let value_json = serde_json::to_vec(&serde_json::Value::String(value.to_string()))
        .context("serializing value")?;
    let req = SetSettingReq {
        key: key.to_string(),
        value_json,
        scope: scope_of(key).into(),
        __buffa_unknown_fields: Default::default(),
    };
    rt.block_on(client.set(req))
        .map_err(|e| anyhow!("setting '{key}' failed: {e}"))?;
    Ok(())
}

pub fn list() -> Result<Vec<(String, String)>> {
    let (client, rt) = connect()?;
    let mut out = Vec::new();
    for scope in [
        SettingScope::SETTING_SCOPE_ACCOUNT,
        SettingScope::SETTING_SCOPE_RUNTIME,
    ] {
        let req = ListSettingsReq {
            scope: scope.into(),
            __buffa_unknown_fields: Default::default(),
        };
        if let Ok(resp) = rt.block_on(client.list(req)) {
            for s in resp.view().settings.iter() {
                out.push((s.key.to_string(), display_value(&s.value_json)));
            }
        }
    }
    out.sort();
    Ok(out)
}
