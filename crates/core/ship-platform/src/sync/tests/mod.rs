/// Integration tests for the sync module.
///
/// The sync module is a thin HTTP transport layer — `SyncClient` + 4 request/
/// response types. It emits 0 events directly (events are emitted by the
/// wrapping `services/sync.rs`). Tests here cover:
///
/// - client construction and URL shaping for all four HTTP methods
/// - type serialisation/deserialisation contracts
/// - wire-format invariants (space_id field placement, field presence)
///
/// Each test is hermetic: `init_project` + `tempdir` per test, no shared state.
/// Event stores are built via `SqliteEventStore::new_at_db_path(&runtime_db_path(&ship_dir))`
/// on an explicit per-scope db, which sidesteps thread-local / env-var
/// isolation issues (cross-module finding).
///
/// Test naming follows `sync_<subdomain>_<scenario>` for Track A coverage tool
/// compatibility.
#[cfg(test)]
mod integration_client;
#[cfg(test)]
mod integration_types;
#[cfg(test)]
mod integration_wire_format;
