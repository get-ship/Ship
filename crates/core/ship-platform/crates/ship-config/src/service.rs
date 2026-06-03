//! ConfigService — settings-tier business logic for the 4 proto RPCs.
//!
//! Reads/writes go through the substrate-owned `daemon_settings` KV port
//! ([`crate::settings`]); the write authors a `config.changed` event through
//! `SpaceContext.kernel()` (the [`KernelLog`] port). No sqlite/raw-fs here.
//!
//! [`KernelLog`]: ship_substrate::ports::KernelLog

use std::sync::Arc;

use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::SpaceContext;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::CONFIG_CHANGED;
use ship_types::payload::ConfigChanged;

use crate::error::ConfigError;
use crate::settings;

/// One resolved setting: key, JSON value, and whether it is a returned default
/// (no stored row).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub is_default: bool,
}

/// Core service — stateless; all state lives in `daemon_settings` + the kernel.
pub struct ConfigService {
    pub ctx: Arc<SpaceContext>,
}

impl ConfigService {
    /// All daemon settings (stored rows merged with the well-known defaults).
    /// A key present in the stored set is `is_default = false`.
    pub fn list(&self) -> Result<Vec<ResolvedSetting>, ConfigError> {
        let stored: std::collections::HashSet<String> = ship_substrate::db::daemon_settings::list_all()
            .map_err(|e| ConfigError::Internal(e.to_string()))?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        let all = settings::get_all_settings().map_err(|e| ConfigError::Internal(e.to_string()))?;
        let mut out: Vec<ResolvedSetting> = all
            .into_iter()
            .map(|(key, value)| {
                let is_default = !stored.contains(&key);
                ResolvedSetting { key, value, is_default }
            })
            .collect();
        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    /// A single setting by key, falling back to its hardcoded default.
    pub fn get(&self, key: &str) -> Result<ResolvedSetting, ConfigError> {
        let is_default = settings::is_default(key).map_err(|e| ConfigError::Internal(e.to_string()))?;
        let value = settings::get_setting(key)
            .map_err(|e| ConfigError::Internal(e.to_string()))?
            .ok_or_else(|| ConfigError::NotFound(format!("setting '{key}' not found")))?;
        Ok(ResolvedSetting { key: key.to_string(), value, is_default })
    }

    /// Set a daemon setting and author exactly one `config.changed` event.
    ///
    /// The event carries `subject = key`, Runtime scope (no space — the config
    /// namespace is daemon-wide), daemon-internal mint. SSE consumers of
    /// `config.*` see the change.
    pub async fn set(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<ResolvedSetting, ConfigError> {
        settings::set_setting(key, value.clone()).map_err(|e| ConfigError::Internal(e.to_string()))?;

        let payload = ConfigChanged { subject: key.to_string(), detail: Some(value.to_string()) };
        let envelope = EventEnvelope::new(CONFIG_CHANGED, key, &payload)
            .map_err(|e| ConfigError::Internal(e.to_string()))?;
        // Config service — its own namespace, no space. Daemon-internal mint.
        let ctx = EmitContext::daemon_internal(None);
        self.ctx.kernel().emit(envelope, &ctx).await?;

        Ok(ResolvedSetting { key: key.to_string(), value, is_default: false })
    }
}
