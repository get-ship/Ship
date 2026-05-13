//! Settings and provider-detection endpoints for the Ship daemon.

use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::auth::{AuthContext, Scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "key": "defaults.provider",
    "value": "claude"
}))]
pub struct SettingsUpdateReq {
    /// Dotted setting key, e.g. `defaults.provider`.
    pub key: String,
    /// Setting value as JSON. Shape depends on the setting (string, bool,
    /// number, or structured object). Validated against the setting's
    /// registered schema before persist.
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
}

#[derive(Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "claude",
    "available": true,
    "binary": "/usr/local/bin/claude",
    "version": "1.0.12"
}))]
pub struct ProviderStatus {
    pub id: String,
    pub available: bool,
    pub binary: String,
    pub version: Option<String>,
}

// ─── Provider cache ─────────────────────────────────────────────────────────

static PROVIDER_CACHE: Mutex<Option<(Instant, Vec<ProviderStatus>)>> = Mutex::new(None);
const PROVIDER_CACHE_TTL: Duration = Duration::from_secs(300);

fn cached_providers() -> Vec<ProviderStatus> {
    let mut guard = match PROVIDER_CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            tracing::warn!("provider cache lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };
    if let Some((ts, ref cached)) = *guard {
        if ts.elapsed() < PROVIDER_CACHE_TTL {
            return cached.clone();
        }
    }
    let providers: Vec<ProviderStatus> = ship_platform::agents::providers::PROVIDERS
        .iter()
        .map(|d| {
            let available = ship_platform::agents::providers::detect_binary(d.binary);
            let version = if available {
                ship_platform::agents::providers::detect_version(d.binary)
            } else {
                None
            };
            ProviderStatus {
                id: d.id.to_string(),
                available,
                binary: d.binary.to_string(),
                version,
            }
        })
        .collect();
    *guard = Some((Instant::now(), providers.clone()));
    providers
}

// ─── Handlers ───────────────────────────────────────────────────────────────

#[utoipa::path(
    operation_id = "listSettings",
    get,
    path = "/api/v1/settings",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "settings"
)]
pub async fn list_settings(
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    match ship_platform::config::get_all_settings() {
        Ok(settings) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!(settings),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

#[utoipa::path(
    operation_id = "updateSetting",
    put,
    path = "/api/v1/settings",
    request_body = SettingsUpdateReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::routes::openapi_types::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 403, description = "insufficient scope", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "settings"
)]
pub async fn update_setting(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Json(req): Json<SettingsUpdateReq>,
) -> impl IntoResponse {
    if !auth.has_scope(Scope::AdminDaemon) {
        return (
            StatusCode::FORBIDDEN,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!("insufficient scope: AdminDaemon required"),
            }),
        );
    }
    match ship_platform::config::set_setting_and_emit(state.kernel(), &req.key, req.value.clone()).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "key": req.key, "value": req.value }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

#[utoipa::path(
    operation_id = "getSetting",
    get,
    path = "/api/v1/settings/{key}",
    params(("key" = String, Path, description = "Setting key (e.g. defaults.provider)")),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 404, description = "setting not found", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "settings"
)]
pub async fn get_setting(
    State(_state): State<ApiState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match ship_platform::config::get_setting(&key) {
        Ok(Some(value)) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "key": key, "value": value }),
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(format!("setting '{}' not found", key)),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

#[utoipa::path(
    operation_id = "listProviders",
    get,
    path = "/api/v1/providers",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "settings"
)]
pub async fn list_providers(
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    let providers = cached_providers();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(providers),
        }),
    )
}
