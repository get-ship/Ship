//! `ship config` over shipd's `SettingsService` (settings.proto). The CLI is a
//! pure client: shipd must be running or these calls error. Scope for `get`/
//! `set` is inferred server-side from the key; the client does not decide it.

use anyhow::{anyhow, Context, Result};
use ship_proto::ship::ops::v1::{GetSettingReq, ListSettingsReq, SetSettingReq, SettingScope, SettingsServiceClient};

use crate::uds_transport::{client_config, connect_transport_or_autostart};

type Transport = crate::uds_transport::UdsTransport;

fn connect() -> Result<(SettingsServiceClient<Transport>, tokio::runtime::Runtime)> {
    let transport = connect_transport_or_autostart().map_err(|e| {
        anyhow!("shipd not running: {e}\nTry: ship daemon start")
    })?;
    let client = SettingsServiceClient::new(transport, client_config());
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
        ..Default::default()
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
        ..Default::default()
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
