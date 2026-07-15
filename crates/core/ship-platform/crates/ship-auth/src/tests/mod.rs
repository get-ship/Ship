/// Integration tests for ship-auth.
///
/// Each test is hermetic: a `tempdir()` is used wherever disk or DB state
/// is needed. Tests that touch the event store call `crate::testutil::init_auth`
/// first, which registers auth projections onto the substrate event bus.
#[cfg(test)]
mod integration_gate_matrix;
#[cfg(test)]
mod integration_session;
#[cfg(test)]
mod integration_attest;
#[cfg(test)]
mod integration_grants;
