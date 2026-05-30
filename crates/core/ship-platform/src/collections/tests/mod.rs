/// Integration tests for the collections module.
///
/// Ephemeral substrate helper: `crate::testutil::init_project(tmp.path())`
/// spins a tmpfs `.ship/` tree with a generated project id and calls
/// `install_defaults()`. `SqliteEventStore::new()` in test context automatically
/// routes to a per-thread temp dir (via `auto_test_global_dir`), so all event
/// writes and reads within a single test are hermetically isolated.
///
/// Note: schema writes (create_schema / update_schema) emit NO events — this
/// is a known gap documented in the module audit. Tests do not assert on
/// schema-write events; only item CRUD events are tested.
///
/// Test naming follows `collections_<subdomain>_<scenario>` so the Track A
/// coverage tool matches them via the `<service>_<method>_*` pattern.
#[cfg(test)]
mod integration_service;
#[cfg(test)]
mod integration_builtin;
#[cfg(test)]
mod integration_schema;
#[cfg(test)]
mod integration_validation;
