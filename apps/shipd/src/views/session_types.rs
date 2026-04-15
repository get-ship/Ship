//! Types used by [`super::session::ViewSessionRegistry`]. Split out to
//! keep `session.rs` under the 300-line cap.

use serde::Serialize;
use std::time::{Duration, Instant};

pub type ViewId = String;
pub type AgentId = String;

#[derive(Debug, Clone, Copy)]
pub struct RegistryConfig {
    pub max_sessions_per_agent: usize,
    pub rate_limit_capacity: f64,
    pub rate_limit_refill_per_sec: f64,
    pub gc_timeout: Duration,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_agent: 3,
            rate_limit_capacity: 20.0,
            rate_limit_refill_per_sec: 20.0,
            gc_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SessionError {
    #[error("view session '{id}' not found")]
    NotFound { id: String },
    #[error("agent '{agent}' does not own session '{id}'")]
    NotOwner { id: String, agent: String },
    #[error("agent '{agent}' at session quota ({limit})")]
    QuotaExceeded { agent: String, limit: usize },
    #[error("rate limit exceeded for session '{id}'")]
    RateLimited { id: String },
    #[error("template '{template}' not found")]
    UnknownTemplate { template: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub view_id: ViewId,
    pub owner_agent_id: AgentId,
    pub template: String,
    pub title: Option<String>,
    pub subscriber_count: usize,
}

/// Token bucket: refills at `refill_per_sec` up to `capacity`.
/// `try_consume` updates tokens from elapsed time then takes `n` if available.
#[derive(Debug)]
pub struct TokenBucket {
    capacity: f64,
    refill_per_sec: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self { capacity, refill_per_sec, tokens: capacity, last_refill: Instant::now() }
    }
    pub fn try_consume(&mut self, n: f64) -> bool {
        let now = Instant::now();
        let dt = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + dt * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;
        if self.tokens + f64::EPSILON >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_consumes_up_to_capacity_then_denies() {
        let mut b = TokenBucket::new(3.0, 1.0);
        assert!(b.try_consume(1.0));
        assert!(b.try_consume(1.0));
        assert!(b.try_consume(1.0));
        assert!(!b.try_consume(1.0), "fourth consume with capacity 3 must fail");
    }

    #[test]
    fn bucket_refills_over_time() {
        let mut b = TokenBucket::new(2.0, 100.0);
        assert!(b.try_consume(2.0));
        assert!(!b.try_consume(1.0));
        std::thread::sleep(std::time::Duration::from_millis(30));
        assert!(b.try_consume(1.0));
    }
}
