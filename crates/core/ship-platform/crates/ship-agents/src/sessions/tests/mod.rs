/// Integration tests for the sessions module.
///
/// Ephemeral substrate helper: `crate::testutil::init_project(tmp.path())`
/// spins a tmpfs `.ship/` tree with a generated project id and calls
/// `install_defaults()`. Event-emitting tests build a store via
/// `SqliteEventStore::new_for_scope(get_global_dir(), Runtime, None)` (or
/// `new_at_db_path` on an explicit per-scope db); the thread-local global dir
/// (via `auto_test_global_dir`) keeps all event writes and reads within a
/// single test hermetically isolated.
///
/// Test naming follows `sessions_<subdomain>_<scenario>` so the Track A coverage
/// tool matches them via the `<service>_<method>_*` pattern.
///
/// These tests DO NOT duplicate `tests_session.rs`, `tests_session_lifecycle.rs`,
/// or `tests_session_events.rs` (inline test files covering lifecycle and events).
/// They cover: helpers (pure fns), types (Display/FromStr/Liveness), artifacts
/// (on-disk file side effects), and list/query edge cases.
#[cfg(test)]
mod integration_helpers;
#[cfg(test)]
mod integration_types;
#[cfg(test)]
mod integration_artifacts;
#[cfg(test)]
mod integration_list_query;
