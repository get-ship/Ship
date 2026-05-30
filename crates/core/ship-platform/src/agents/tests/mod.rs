/// Integration tests for the agents module.
///
/// Ephemeral substrate helper: `crate::testutil::init_project(tmp.path())`
/// spins a tmpfs `.ship/` tree with a generated project id and calls
/// `install_defaults()`. `SqliteEventStore::new()` in test context automatically
/// routes to a per-thread temp dir (via `auto_test_global_dir`), so all event
/// writes and reads within a single test are hermetically isolated.
///
/// Test naming follows `agents_<subdomain>_<scenario>` so the Track A coverage
/// tool matches them via the `<service>_<method>_*` pattern.
#[cfg(test)]
mod integration_skill_resolution;
#[cfg(test)]
mod integration_crud;
#[cfg(test)]
mod integration_profile_resolution;
#[cfg(test)]
mod integration_harness_routing;
