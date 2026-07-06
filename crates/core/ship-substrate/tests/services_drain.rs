//! Tests for the graceful drain contract on ServiceHandler.
//!
//! These tests define the API surface for on_drain and health_check.
//! They are expected to fail until the implementation is added.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;

use ship_substrate::events::{EventEnvelope, Mailbox, SqliteEventStore};
use ship_substrate::services::{run_service, HealthStatus, ServiceHandler};

// ── Fixtures ─────────────────────────────────────────────────────────────────

struct MinimalHandler;

impl ServiceHandler for MinimalHandler {
    fn name(&self) -> &str { "minimal" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }
}

struct TimeoutRecorder {
    recorded: Arc<std::sync::Mutex<Option<Duration>>>,
}

impl ServiceHandler for TimeoutRecorder {
    fn name(&self) -> &str { "timeout-recorder" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }
    fn on_drain(&mut self, timeout: Duration) -> Result<()> {
        *self.recorded.lock().unwrap() = Some(timeout);
        Ok(())
    }
}

struct DrainOrderRecorder {
    calls: Arc<std::sync::Mutex<Vec<String>>>,
}

impl ServiceHandler for DrainOrderRecorder {
    fn name(&self) -> &str { "drain-order" }
    fn handle(&mut self, _event: &EventEnvelope, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }
    fn on_drain(&mut self, _timeout: Duration) -> Result<()> {
        self.calls.lock().unwrap().push("on_drain".into());
        Ok(())
    }
    fn on_stop(&mut self, _store: &SqliteEventStore) -> Result<()> {
        self.calls.lock().unwrap().push("on_stop".into());
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

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
    let recorded = Arc::new(std::sync::Mutex::new(None));
    let mut handler = TimeoutRecorder { recorded: recorded.clone() };
    handler.on_drain(Duration::from_secs(30)).unwrap();
    let value = recorded.lock().unwrap();
    assert_eq!(*value, Some(Duration::from_secs(30)));
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_service_calls_on_drain_when_mailbox_closes() {
    let (tx, rx) = mpsc::channel(4);
    let mailbox = Mailbox::from_receiver(rx);

    let tmp = tempfile::tempdir().unwrap();
    let db_path = ship_substrate::db::runtime_db_path(tmp.path());
    ship_substrate::db::ensure_events_db_at_path(&db_path).unwrap();
    let store = SqliteEventStore::new_at_db_path(&db_path).unwrap();

    let calls: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let handler = DrainOrderRecorder { calls: calls.clone() };
    let task = tokio::spawn(run_service(Box::new(handler), store, mailbox));

    drop(tx);
    task.await.unwrap();

    let calls = calls.lock().unwrap();
    // The runner must call on_drain BEFORE on_stop.
    let on_drain_idx = calls.iter().position(|c| c == "on_drain");
    let on_stop_idx = calls.iter().position(|c| c == "on_stop");
    assert!(on_drain_idx.is_some(), "on_drain must be called");
    assert!(on_stop_idx.is_some(), "on_stop must be called");
    assert!(on_drain_idx < on_stop_idx, "on_drain must precede on_stop");
    let _ = tmp; // keep tmp alive
}
