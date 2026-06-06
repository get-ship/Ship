//! Settings and harness-detection endpoints for the Ship daemon.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::routes::rest_api::{ApiState, MeshResponse};

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct ProviderStatus {
    pub id: String,
    pub available: bool,
    pub binary: String,
    pub version: Option<String>,
}

// ─── Harness cache ─────────────────────────────────────────────────────────

static HARNESS_CACHE: Mutex<Option<(Instant, Vec<ProviderStatus>)>> = Mutex::new(None);
const HARNESS_CACHE_TTL: Duration = Duration::from_secs(300);

fn cached_harnesses() -> Vec<ProviderStatus> {
    let mut guard = match HARNESS_CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            tracing::warn!("harness cache lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };
    if let Some((ts, ref cached)) = *guard {
        if ts.elapsed() < HARNESS_CACHE_TTL {
            return cached.clone();
        }
    }
    let harnesses: Vec<ProviderStatus> = ship_agents::agent_domain::harnesses::HARNESSES
        .iter()
        .map(|d| {
            let available = ship_agents::agent_domain::harnesses::detect_binary(d.binary);
            let version = if available {
                ship_agents::agent_domain::harnesses::detect_version(d.binary)
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
    *guard = Some((Instant::now(), harnesses.clone()));
    harnesses
}

// ─── Handlers ───────────────────────────────────────────────────────────────


pub async fn list_harnesses(
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    let harnesses = cached_harnesses();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(harnesses),
        }),
    )
}
