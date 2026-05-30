/// Integration tests for the artifacts module.
///
/// Ephemeral substrate: each test calls `setup()` which spins its own
/// `tempdir()`, `ArtifactService`, `DataService`, and `KernelRouter`.
/// Event assertions query the per-thread auto-isolated `SqliteEventStore`
/// via `query_events()`.
///
/// Test naming follows `artifacts_<subdomain>_<scenario>` so the Track A
/// coverage tool matches them via the `<service>_<method>_*` pattern.
#[cfg(test)]
mod integration_store;
#[cfg(test)]
mod integration_promote;
#[cfg(test)]
mod integration_gc;
#[cfg(test)]
mod integration_backend;
