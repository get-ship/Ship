/// Integration tests for the auth module.
///
/// Naming follows `auth_<subdomain>_<scenario>` so the Track A coverage
/// tool matches them via the `<service>_<method>_*` pattern.
///
/// Each test is hermetic: a `tempdir()` is used wherever disk or DB state
/// is needed. Event-emitting tests build a store via
/// `SqliteEventStore::new_for_scope(get_global_dir(), Runtime, None)` (or
/// `new_at_db_path` on an explicit per-scope db), which routes to the
/// per-thread auto-isolated store (via `auto_test_global_dir`). Tests that
/// touch the event store call `init_project` first.
#[cfg(test)]
mod integration_gate_matrix;
#[cfg(test)]
mod integration_pairing;
#[cfg(test)]
mod integration_session;
#[cfg(test)]
mod integration_attest;
#[cfg(test)]
mod integration_grants;
