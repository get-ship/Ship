//! Agent-owned view sessions.
//!
//! `ViewSessionRegistry` tracks live canvas sessions opened by agents via the
//! `view.open` MCP tool. Each session has an owner `agent_id`, a template, a
//! live set of `ViewStream` subscribers, and a per-session token bucket.
//! Sessions are GC'd `cfg.gc_timeout` after the last subscriber disconnects.
//!
//! Types are split into `session_types.rs` to stay under the 300-line cap.
//! See `.artifacts/phase-1-agent-canvas-views-spec.md` for the full contract.

use super::session_types::{RegistryConfig, SessionError, SessionInfo, TokenBucket, ViewId};
use super::stream::{ViewSse, ViewStream};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use ulid::Ulid;

pub use super::session_types::{AgentId, RegistryConfig as Config, SessionError as Error, SessionInfo as Info};

pub(crate) struct SessionEntry {
    pub(crate) owner_agent_id: AgentId,
    pub(crate) template: String,
    pub(crate) title: Option<String>,
    pub(crate) context: Value,
    pub(crate) subscribers: Vec<ViewStream>,
    pub(crate) last_disconnect_at: Option<Instant>,
    pub(crate) tokens: TokenBucket,
}

pub struct ViewSessionRegistry {
    cfg: RegistryConfig,
    sessions: Mutex<HashMap<ViewId, SessionEntry>>,
}

pub type SharedViewSessionRegistry = Arc<ViewSessionRegistry>;

impl Default for ViewSessionRegistry {
    fn default() -> Self {
        Self::with_config(RegistryConfig::default())
    }
}

impl ViewSessionRegistry {
    pub fn with_config(cfg: RegistryConfig) -> Self {
        Self { cfg, sessions: Mutex::new(HashMap::new()) }
    }

    pub fn config(&self) -> RegistryConfig {
        self.cfg
    }

    pub async fn open(
        &self,
        owner: AgentId,
        template: String,
        title: Option<String>,
        context: Value,
    ) -> Result<ViewId, SessionError> {
        let mut s = self.sessions.lock().await;
        Self::gc_locked(&self.cfg, &mut s);
        let existing = s.values().filter(|e| e.owner_agent_id == owner).count();
        if existing >= self.cfg.max_sessions_per_agent {
            return Err(SessionError::QuotaExceeded {
                agent: owner,
                limit: self.cfg.max_sessions_per_agent,
            });
        }
        let view_id = Ulid::new().to_string();
        s.insert(
            view_id.clone(),
            SessionEntry {
                owner_agent_id: owner,
                template,
                title,
                context,
                subscribers: Vec::new(),
                last_disconnect_at: Some(Instant::now()),
                tokens: TokenBucket::new(
                    self.cfg.rate_limit_capacity,
                    self.cfg.rate_limit_refill_per_sec,
                ),
            },
        );
        Ok(view_id)
    }

    pub async fn close(&self, id: &str, caller: &str) -> Result<(), SessionError> {
        let mut s = self.sessions.lock().await;
        let entry = s
            .get(id)
            .ok_or_else(|| SessionError::NotFound { id: id.to_string() })?;
        if entry.owner_agent_id != caller {
            return Err(SessionError::NotOwner { id: id.to_string(), agent: caller.to_string() });
        }
        s.remove(id);
        Ok(())
    }

    pub async fn subscribe(&self, id: &str) -> Result<ViewSse, SessionError> {
        let mut s = self.sessions.lock().await;
        let entry = s
            .get_mut(id)
            .ok_or_else(|| SessionError::NotFound { id: id.to_string() })?;
        let (producer, sse) = ViewStream::new();
        entry.subscribers.push(producer);
        entry.last_disconnect_at = None;
        Ok(sse)
    }

    pub async fn shell_context(
        &self,
        id: &str,
    ) -> Result<(String, Option<String>, Value), SessionError> {
        let s = self.sessions.lock().await;
        let entry = s
            .get(id)
            .ok_or_else(|| SessionError::NotFound { id: id.to_string() })?;
        Ok((entry.template.clone(), entry.title.clone(), entry.context.clone()))
    }

    pub async fn patch(
        &self,
        id: &str,
        caller: &str,
        selector: &str,
        html: String,
    ) -> Result<(), SessionError> {
        self.authorize(id, caller).await?;
        self.broadcast(id, Broadcast::Patch(selector.to_string(), html)).await
    }

    pub async fn append(
        &self,
        id: &str,
        caller: &str,
        selector: &str,
        html: String,
    ) -> Result<(), SessionError> {
        self.authorize(id, caller).await?;
        // Datastar append mode (ElementPatchMode::After) is not yet wired
        // through ViewStream; Phase 1 delivers as a morph patch against the
        // same selector. Upstream template authors use `<ship-feed>` or an
        // explicit list + append-shaped selector.
        self.broadcast(id, Broadcast::Patch(selector.to_string(), html)).await
    }

    pub async fn signal(
        &self,
        id: &str,
        caller: &str,
        name: &str,
        value: Value,
    ) -> Result<(), SessionError> {
        self.authorize(id, caller).await?;
        let json = serde_json::json!({ name: value }).to_string();
        self.broadcast(id, Broadcast::Signal(json)).await
    }

    pub async fn list(&self) -> Vec<SessionInfo> {
        let s = self.sessions.lock().await;
        s.iter()
            .map(|(id, e)| SessionInfo {
                view_id: id.clone(),
                owner_agent_id: e.owner_agent_id.clone(),
                template: e.template.clone(),
                title: e.title.clone(),
                subscriber_count: e.subscribers.iter().filter(|x| !x.is_closed()).count(),
            })
            .collect()
    }

    pub async fn gc_now(&self) {
        let mut s = self.sessions.lock().await;
        Self::gc_locked(&self.cfg, &mut s);
    }

    fn gc_locked(cfg: &RegistryConfig, sessions: &mut HashMap<ViewId, SessionEntry>) {
        let now = Instant::now();
        for entry in sessions.values_mut() {
            entry.subscribers.retain(|s| !s.is_closed());
            match (entry.subscribers.is_empty(), entry.last_disconnect_at) {
                (true, None) => entry.last_disconnect_at = Some(now),
                (false, _) => entry.last_disconnect_at = None,
                _ => {}
            }
        }
        sessions.retain(|_, e| match e.last_disconnect_at {
            Some(t) if now.duration_since(t) > cfg.gc_timeout => false,
            _ => true,
        });
    }

    async fn authorize(&self, id: &str, caller: &str) -> Result<(), SessionError> {
        let mut s = self.sessions.lock().await;
        let entry = s
            .get_mut(id)
            .ok_or_else(|| SessionError::NotFound { id: id.to_string() })?;
        if entry.owner_agent_id != caller {
            return Err(SessionError::NotOwner { id: id.to_string(), agent: caller.to_string() });
        }
        if !entry.tokens.try_consume(1.0) {
            return Err(SessionError::RateLimited { id: id.to_string() });
        }
        Ok(())
    }

    async fn broadcast(&self, id: &str, what: Broadcast) -> Result<(), SessionError> {
        let mut s = self.sessions.lock().await;
        let entry = s
            .get_mut(id)
            .ok_or_else(|| SessionError::NotFound { id: id.to_string() })?;
        let mut live = Vec::with_capacity(entry.subscribers.len());
        for sub in entry.subscribers.drain(..) {
            let ok = match &what {
                Broadcast::Patch(sel, html) => {
                    if sel.is_empty() {
                        sub.patch_html(html.clone()).await
                    } else {
                        sub.patch(sel, html.clone()).await
                    }
                }
                Broadcast::Signal(json) => sub.signal(json.clone()).await,
            };
            if ok.is_ok() {
                live.push(sub);
            }
        }
        entry.subscribers = live;
        Ok(())
    }
}

enum Broadcast {
    Patch(String, String),
    Signal(String),
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
