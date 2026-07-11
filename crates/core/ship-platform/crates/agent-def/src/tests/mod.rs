/// Integration tests for the agents module.
///
/// Ephemeral substrate helper: `crate::testutil::init_project(tmp.path())`
/// spins a tmpfs `.ship/` tree with a generated project id and calls
/// `install_defaults()`. Event-emitting tests build a store via
/// `SqliteEventStore::new_for_scope(get_global_dir(), Runtime, None)` (or
/// `new_at_db_path` on an explicit per-scope db); the thread-local global dir
/// (via `auto_test_global_dir`) keeps all event writes and reads within a
/// single test hermetically isolated.
///
/// Test naming follows `agents_<subdomain>_<scenario>` so the Track A coverage
/// tool matches them via the `<service>_<method>_*` pattern.
#[cfg(test)]
mod integration_skill_resolution;
#[cfg(test)]
mod integration_harness_routing;
#[cfg(test)]
mod event_scope_discipline;
#[cfg(test)]
mod file_changed_consumer;
