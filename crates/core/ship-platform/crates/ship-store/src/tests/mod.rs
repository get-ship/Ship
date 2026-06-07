/// Integration tests for the registry module.
///
/// Each test uses a fresh `tempdir()` — no shared mutable state. The registry
/// module emits 0 events (verified by the module audit), so there are no event
/// assertions here; tests verify return values and filesystem state produced by
/// the pipeline.
///
/// Test naming follows `registry_<subdomain>_<scenario>` so the Track A
/// coverage tool matches them via the `<service>_<method>_*` pattern.
#[cfg(test)]
mod integration_hash;
#[cfg(test)]
mod integration_cache;
#[cfg(test)]
mod integration_dep_skills;
#[cfg(test)]
mod integration_compose;
#[cfg(test)]
mod integration_constraint;
