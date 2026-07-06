/// Integration tests for the triggers module.
///
/// Ephemeral substrate: `tmp_ship_dir()` spins a tmpfs `.ship/` tree with an
/// initialised SQLite event store. Each test writes its own trigger JSONC files
/// so every case is hermetically isolated.
///
/// Test naming follows `triggers_<subdomain>_<scenario>` so the Track A
/// coverage tool matches them via the `<service>_<method>_*` pattern.
mod descriptor_conversion;
mod integration_fan_out;
mod integration_filter;
mod integration_reload;
mod integration_engine;
