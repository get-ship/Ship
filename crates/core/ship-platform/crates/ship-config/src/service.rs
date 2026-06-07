//! `SettingsService` — the unified, scope-discriminated settings core.
//!
//! One service over three scopes. Each `Set` writes the read-model row in the
//! matching per-scope store AND authors a `settings.changed` event in that
//! scope, so the kernel routes the event to the matching per-scope db:
//!
//! - **RUNTIME** → the `daemon_settings` KV (`runtime.db`) via [`crate::settings`];
//!   `settings.changed` rides `EventScope::Runtime`. Machine-local launch defaults.
//! - **ACCOUNT** → the `settings_overlay` table in `account.db` (a per-account
//!   `DataStore`); `settings.changed` rides `EventScope::Account`.
//! - **SPACE** → the `settings_overlay` table in the ACTIVE space's `space.db`
//!   (a per-space `DataStore`); `settings.changed` rides `EventScope::Space`,
//!   keyed by the space id. Errors cleanly when no space is active.
//!
//! Authority gate (restrictive default; the owner/contributor + agent-subset
//! role model is a flagged follow): SPACE writes are **operator-only** — agents
//! cannot write space settings. ACCOUNT/RUNTIME writes are operator/shell-cred.
//! The emit itself is daemon-mediated (the daemon is the trusted intermediary
//! that mints a daemon-internal binding carrying the chosen scope key), mirror-
//! ing `SpaceService.SetSetting`.
//!
//! No sqlite/raw-fs here — storage goes through the substrate ports + the
//! `daemon_settings` accessor; events through `KernelLog`.

use std::sync::Arc;

use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::{DataStore, KernelLog};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use ship_types::event_types::SETTINGS_CHANGED;
use ship_types::payload::SettingChanged;

use crate::error::ConfigError;
use crate::{overlay, settings};

/// The settings scope discriminant — the crate's mirror of the proto
/// `SettingScope`. The wire maps the proto enum into this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingScope {
    /// Machine-local launch defaults (`runtime.db` / `daemon_settings`).
    Runtime,
    /// The user's account (`account.db`).
    Account,
    /// The connection's active space (`space.db` overlay).
    Space,
}

impl SettingScope {
    /// The matching event partition axis. The chosen scope rides the envelope.
    fn event_scope(self) -> EventScope {
        match self {
            SettingScope::Runtime => EventScope::Runtime,
            SettingScope::Account => EventScope::Account,
            SettingScope::Space => EventScope::Space,
        }
    }

    /// The human-readable echo carried in the `SettingChanged` payload + the
    /// resolved `Setting.scope` (matches `EventScope::as_str`).
    pub fn as_str(self) -> &'static str {
        self.event_scope().as_str()
    }
}

/// One resolved setting: key, JSON value, the scope it came from, and whether it
/// is a returned default (no stored row).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSetting {
    pub key: String,
    pub value: serde_json::Value,
    pub scope: SettingScope,
    pub is_default: bool,
}

/// Per-request scope handles the daemon resolves and hands the service.
///
/// `kernel` is the process-global router handle (one suffices — the kernel
/// routes by the event's `EventScope`, not by which handle). The Account store
/// binds `account.db`; the optional Space binding carries the active space id +
/// its `space.db` store (absent ⟹ no active space). `is_operator` is the
/// caller's operator-class verdict (the authority gate input).
///
/// `Clone` is cheap (Arc handles + a small space tuple) so the Watch resolver
/// can own a snapshot copy per subscription.
#[derive(Clone)]
pub struct SettingsCtx {
    pub kernel: Arc<KernelLog>,
    pub account_store: Arc<dyn DataStore>,
    pub space: Option<(String, Arc<dyn DataStore>)>,
    pub is_operator: bool,
}

/// Core service — stateless; all state lives in the per-scope stores + kernel.
pub struct SettingsService {
    pub ctx: SettingsCtx,
}

impl SettingsService {
    pub fn new(ctx: SettingsCtx) -> Self {
        Self { ctx }
    }

    /// All settings in `scope` (stored rows merged with defaults where the scope
    /// has hardcoded defaults). RUNTIME merges the well-known infra defaults;
    /// the overlay scopes (ACCOUNT/SPACE) have no hardcoded defaults — they
    /// return exactly their stored rows.
    pub async fn list(&self, scope: SettingScope) -> Result<Vec<ResolvedSetting>, ConfigError> {
        let mut out = match scope {
            SettingScope::Runtime => self.list_runtime()?,
            SettingScope::Account => self.list_overlay(scope, self.ctx.account_store.as_ref()).await?,
            SettingScope::Space => {
                let store = self.space_store()?;
                self.list_overlay(scope, store).await?
            }
        };
        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    /// A single setting by key in `scope`. RUNTIME falls back to the hardcoded
    /// default; the overlay scopes error `NotFound` for an unset key.
    pub async fn get(&self, scope: SettingScope, key: &str) -> Result<ResolvedSetting, ConfigError> {
        match scope {
            SettingScope::Runtime => {
                let is_default =
                    settings::is_default(key).map_err(|e| ConfigError::Internal(e.to_string()))?;
                let value = settings::get_setting(key)
                    .map_err(|e| ConfigError::Internal(e.to_string()))?
                    .ok_or_else(|| ConfigError::NotFound(format!("setting '{key}' not found")))?;
                Ok(ResolvedSetting { key: key.to_string(), value, scope, is_default })
            }
            SettingScope::Account => self.get_overlay(scope, self.ctx.account_store.as_ref(), key).await,
            SettingScope::Space => {
                let store = self.space_store()?;
                self.get_overlay(scope, store, key).await
            }
        }
    }

    /// Set a setting in `scope`: enforce the authority gate, persist the
    /// read-model row in the matching per-scope store, then author exactly one
    /// `settings.changed` event riding that scope.
    pub async fn set(
        &self,
        scope: SettingScope,
        key: &str,
        value: serde_json::Value,
    ) -> Result<ResolvedSetting, ConfigError> {
        self.authorize_write(scope)?;

        // 1. Persist the read-model row in the scope's store.
        match scope {
            SettingScope::Runtime => {
                settings::set_setting(key, value.clone())
                    .map_err(|e| ConfigError::Internal(e.to_string()))?;
            }
            SettingScope::Account => {
                overlay::set(self.ctx.account_store.as_ref(), key, &value).await?;
            }
            SettingScope::Space => {
                let store = self.space_store()?;
                overlay::set(store, key, &value).await?;
            }
        }

        // 2. Author the scope-discriminated event (daemon-mediated mint).
        self.emit_changed(scope, key, &value).await?;

        Ok(ResolvedSetting { key: key.to_string(), value, scope, is_default: false })
    }

    /// The Watch snapshot for `scope`: `{ "settings": { key: value } }`. The
    /// resolver diffs successive snapshots into patch frames. Mirrors the shape
    /// `list` returns so a subscriber sees the same view. A scope that cannot be
    /// read (e.g. SPACE with no active space) snapshots empty.
    pub async fn snapshot(&self, scope: SettingScope) -> serde_json::Value {
        let entries = self.list(scope).await.unwrap_or_default();
        let obj: serde_json::Map<String, serde_json::Value> =
            entries.into_iter().map(|r| (r.key, r.value)).collect();
        serde_json::json!({ "settings": obj })
    }

    // ── internals ─────────────────────────────────────────────────────────────

    /// The authority gate. SPACE = operator-only (agents cannot write space
    /// settings). ACCOUNT/RUNTIME = operator/shell-cred. Restrictive default; the
    /// finer role model is a flagged follow.
    fn authorize_write(&self, scope: SettingScope) -> Result<(), ConfigError> {
        if self.ctx.is_operator {
            return Ok(());
        }
        let _ = scope; // all non-operator writes are rejected under the default.
        Err(ConfigError::Forbidden(format!(
            "settings.write in {} scope requires operator authority",
            scope.as_str()
        )))
    }

    /// Resolve the active-space read-model store, or error cleanly when there is
    /// no active space (post-S2 there is no `global` fallback).
    fn space_store(&self) -> Result<&dyn DataStore, ConfigError> {
        self.ctx
            .space
            .as_ref()
            .map(|(_, store)| store.as_ref())
            .ok_or_else(|| ConfigError::NotFound("no active space for SPACE-scoped settings".into()))
    }

    fn list_runtime(&self) -> Result<Vec<ResolvedSetting>, ConfigError> {
        let stored: std::collections::HashSet<String> = ship_substrate::db::daemon_settings::list_all()
            .map_err(|e| ConfigError::Internal(e.to_string()))?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        let all = settings::get_all_settings().map_err(|e| ConfigError::Internal(e.to_string()))?;
        Ok(all
            .into_iter()
            .map(|(key, value)| ResolvedSetting {
                key: key.clone(),
                value,
                scope: SettingScope::Runtime,
                is_default: !stored.contains(&key),
            })
            .collect())
    }

    async fn list_overlay(
        &self,
        scope: SettingScope,
        store: &dyn DataStore,
    ) -> Result<Vec<ResolvedSetting>, ConfigError> {
        let map = overlay::get_all(store).await?;
        Ok(map
            .into_iter()
            .map(|(key, value)| ResolvedSetting { key, value, scope, is_default: false })
            .collect())
    }

    async fn get_overlay(
        &self,
        scope: SettingScope,
        store: &dyn DataStore,
        key: &str,
    ) -> Result<ResolvedSetting, ConfigError> {
        let value = overlay::get(store, key)
            .await?
            .ok_or_else(|| ConfigError::NotFound(format!("setting '{key}' not found in {} scope", scope.as_str())))?;
        Ok(ResolvedSetting { key: key.to_string(), value, scope, is_default: false })
    }

    /// Author exactly one `settings.changed` event riding `scope`. SPACE carries
    /// the active space id on the binding so `enforce_scope` stamps + routes it
    /// to `space.db`; ACCOUNT/RUNTIME carry no space key.
    async fn emit_changed(
        &self,
        scope: SettingScope,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), ConfigError> {
        let payload = SettingChanged {
            key: key.to_string(),
            value_json: value.to_string(),
            scope: scope.as_str().to_string(),
        };
        let envelope = EventEnvelope::new(SETTINGS_CHANGED, key, &payload)
            .map_err(|e| ConfigError::Internal(e.to_string()))?
            .with_scope(scope.event_scope(), None);

        let emit_ctx = match scope {
            SettingScope::Space => {
                let (space_id, _) = self.ctx.space.as_ref().ok_or_else(|| {
                    ConfigError::NotFound("no active space for SPACE-scoped settings".into())
                })?;
                EmitContext::daemon_internal(Some(SpaceId::new(space_id.clone())))
            }
            SettingScope::Account | SettingScope::Runtime => EmitContext::daemon_internal(None),
        };
        self.ctx.kernel.emit(envelope, &emit_ctx).await?;
        Ok(())
    }
}
