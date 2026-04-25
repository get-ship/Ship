//! MCP-side `emit_domain_event` is pure transport — its only job is to
//! forward the agent-supplied fields plus injected actor/workspace context
//! to the daemon's `/api/events/emit` endpoint with `validate_domain=true`.
//!
//! Validation (reserved namespaces, dotted event type, payload shape) lives
//! on the daemon and is exercised by `apps/shipd/src/project_api/tests.rs`.
//! We don't unit-test the wire helper here because it requires a running
//! daemon to make any assertion meaningful — covered by integration tests.

#[test]
fn module_is_transport_only() {
    // This file deliberately contains no `EventEnvelope::new` or
    // `SqliteEventStore` references — that's the whole point of the
    // shadowing fix. The structural test (rg) lives in CI; this stub
    // keeps the test module attached and documents intent.
}
