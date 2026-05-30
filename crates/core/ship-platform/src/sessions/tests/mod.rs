/// Integration tests for the sessions module.
///
/// Ephemeral substrate helper: `crate::testutil::init_project(tmp.path())`
/// spins a tmpfs `.ship/` tree with a generated project id and calls
/// `install_defaults()`. `SqliteEventStore::new()` in test context automatically
/// routes to a per-thread temp dir (via `auto_test_global_dir`), so all event
/// writes and reads within a single test are hermetically isolated.
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
