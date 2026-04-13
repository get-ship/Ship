//! Tests for the graceful drain contract on ServiceHandler.
//!
//! These tests define the API surface for on_drain and health_check.
//! They are expected to fail until the implementation is added.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tempfile::tempdir;
use tokio::sync::mpsc;

use crate::events::{ActorStore, EventEnvelope, Mailbox};
use crate::services::{run_service, HealthStatus, ServiceHandler};

// ── Fixtures ────────────────────────────────────────────────────────────────

struct MinimalHandler;

impl ServiceHandler for MinimalHandler {
    fn name(&self) -> &str { "minimal" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        Ok(())
    }
}

struct TimeoutRecorder {
    recorded: Arc<Mutex<Option<Duration>>>,
}

impl ServiceHandler for TimeoutRecorder {
    fn name(&self) -> &str { "timeout-recorder" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        Ok(())
    }
    fn on_drain(&mut self, timeout: Duration) -> Result<()> {
        *self.recorded.lock().unwrap() = Some(timeout);
        Ok(())
    }
}

struct DrainOrderRecorder {
    calls: Arc<Mutex<Vec<String>>>,
}

impl ServiceHandler for DrainOrderRecorder {
    fn name(&self) -> &str { "drain-order" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        Ok(())
    }
    fn on_drain(&mut self, _timeout: Duration) -> Result<()> {
        self.calls.lock().unwrap().push("on_drain".into());
        Ok(())
    }
    fn on_stop(&mut self, _store: &ActorStore) -> Result<()> {
        self.calls.lock().unwrap().push("on_stop".into());
        Ok(())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn service_handler_has_on_drain_default() {
    let mut handler = MinimalHandler;
    let result = handler.on_drain(Duration::from_secs(10));
    assert!(result.is_ok(), "default on_drain should return Ok");
}

#[test]
fn service_handler_has_health_check_default() {
    let handler = MinimalHandler;
    let status = handler.health_check();
    assert!(
        matches!(status, HealthStatus::Healthy),
        "default health_check should return Healthy"
    );
}

#[test]
fn health_status_variants_exist() {
    let _healthy = HealthStatus::Healthy;
    let _degraded = HealthStatus::Degraded { reason: "test".into() };
    let _unhealthy = HealthStatus::Unhealthy { reason: "test".into() };
}

#[test]
fn on_drain_receives_timeout() {
    let recorded = Arc::new(Mutex::new(None));
    let mut handler = TimeoutRecorder { recorded: recorded.clone() };
    handler.on_drain(Duration::from_secs(30)).unwrap();
    let value = recorded.lock().unwrap();
    assert_eq!(*value, Some(Duration::from_secs(30)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_service_calls_on_drain_when_mailbox_closes() {
    let (tx, rx) = mpsc::channel(4);
    let mailbox = Mailbox::from_receiver(rx);

    let tmp = tempdir().unwrap();
    crate::db::ensure_db_at(tmp.path()).unwrap();
    let store = ActorStore::new(
        "drain-svc",
        tmp.path().join("platform.db"),
        vec!["test.".into()],
        vec![],
    );

    let calls = Arc::new(Mutex::new(Vec::new()));
    let handler = DrainOrderRecorder { calls: calls.clone() };

    let task = tokio::spawn(run_service(Box::new(handler), store, mailbox));

    // Close the mailbox — simulates shutdown
    drop(tx);
    tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .expect("service did not exit within timeout")
        .unwrap();

    let calls = calls.lock().unwrap();
    assert!(calls.contains(&"on_drain".to_string()), "on_drain must be called");
    assert!(calls.contains(&"on_stop".to_string()), "on_stop must be called");

    let drain_pos = calls.iter().position(|c| c == "on_drain").unwrap();
    let stop_pos = calls.iter().position(|c| c == "on_stop").unwrap();
    assert!(drain_pos < stop_pos, "on_drain must be called before on_stop");
}
