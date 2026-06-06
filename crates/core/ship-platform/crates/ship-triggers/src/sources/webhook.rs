//! Webhook trigger source adapter.
//!
//! Maintains a registry of path → hook mappings. The shipd router mounts a
//! public POST handler at `/api/hooks/webhook/*path` that calls
//! `WebhookSource::handle_request` for every incoming request.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use serde_json::json;
use tokio::sync::Mutex;

use ship_substrate::events::{EmitContext, EventEnvelope, KernelRouter};

use super::super::def::{HookDef, TriggerSourceDef};
use super::TriggerSource;

/// Max requests per minute per registered path (token bucket capacity).
const RATE_LIMIT_PER_MIN: u32 = 60;

struct RegisteredWebhook {
    hook_name: String,
    secret: Option<String>,
    /// Token bucket: (tokens, last_refill_instant).
    bucket: std::sync::Mutex<(u32, Instant)>,
}

impl RegisteredWebhook {
    fn new(hook_name: String, secret: Option<String>) -> Self {
        Self {
            hook_name,
            secret,
            bucket: std::sync::Mutex::new((RATE_LIMIT_PER_MIN, Instant::now())),
        }
    }

    fn consume_token(&self) -> bool {
        let mut b = self.bucket.lock().unwrap();
        let elapsed_secs = b.1.elapsed().as_secs_f64();
        let refill = (elapsed_secs * RATE_LIMIT_PER_MIN as f64 / 60.0) as u32;
        if refill > 0 {
            b.0 = (b.0 + refill).min(RATE_LIMIT_PER_MIN);
            b.1 = Instant::now();
        }
        if b.0 > 0 {
            b.0 -= 1;
            true
        } else {
            false
        }
    }
}

/// Outcome of a webhook request.
pub enum WebhookResult {
    Accepted,
    NotFound,
    Unauthorized,
    RateLimited,
}

/// Thread-safe registry of webhook triggers.
pub struct WebhookSource {
    webhooks: RwLock<HashMap<String, Arc<RegisteredWebhook>>>,
}

impl WebhookSource {
    pub fn new() -> Self {
        Self { webhooks: RwLock::new(HashMap::new()) }
    }

    /// Handle an inbound webhook POST. Called by the shipd axum handler.
    pub async fn handle_request(
        &self,
        path: &str,
        secret_header: Option<&str>,
        body: Vec<u8>,
        kernel: &Arc<Mutex<KernelRouter>>,
    ) -> WebhookResult {
        let entry = {
            let guard = self.webhooks.read().unwrap();
            guard.get(path).cloned()
        };

        let webhook = match entry {
            Some(w) => w,
            None => {
                tracing::warn!(path, "webhook: no trigger registered for path");
                return WebhookResult::NotFound;
            }
        };

        if let Some(expected) = &webhook.secret {
            match secret_header {
                Some(h) if constant_time_eq(h, expected) => {}
                _ => return WebhookResult::Unauthorized,
            }
        }

        if !webhook.consume_token() {
            return WebhookResult::RateLimited;
        }

        let payload_value =
            serde_json::from_slice::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null);
        let event_type = format!("external.webhook.{}", webhook.hook_name);
        let payload = json!({ "path": path, "body": payload_value });

        match EventEnvelope::new(&event_type, path, &payload) {
            Ok(envelope) => {
                // Trigger source: no resolved space → daemon-internal, keyless.
                let ctx = EmitContext::daemon_internal(None);
                if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
                    tracing::warn!(path, "webhook: emit failed: {e}");
                }
            }
            Err(e) => tracing::warn!(path, "webhook: build envelope failed: {e}"),
        }

        WebhookResult::Accepted
    }
}

impl TriggerSource for WebhookSource {
    fn start(&self, _kernel: Arc<Mutex<KernelRouter>>) -> tokio::task::JoinHandle<()> {
        // No background loop needed — the axum handler is the entry point.
        tokio::spawn(std::future::ready(()))
    }

    fn reload(&self, new_defs: &[HookDef]) {
        let mut map = HashMap::new();
        for def in new_defs {
            if let Some(TriggerSourceDef::Webhook(w)) = &def.trigger.source {
                let secret = w.secret.as_deref().map(resolve_secret);
                map.insert(
                    w.path.clone(),
                    Arc::new(RegisteredWebhook::new(def.name.clone(), secret)),
                );
            }
        }
        if let Ok(mut guard) = self.webhooks.write() {
            *guard = map;
        }
    }
}

/// Resolve a secret reference: `env:VAR_NAME` reads from the environment;
/// anything else is used literally.
fn resolve_secret(raw: &str) -> String {
    if let Some(var) = raw.strip_prefix("env:") {
        std::env::var(var).unwrap_or_else(|_| {
            tracing::warn!(var, "webhook: env var for secret not set");
            String::new()
        })
    } else {
        raw.to_string()
    }
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}
