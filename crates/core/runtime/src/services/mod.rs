// Cross-component integration tests for the service layer. The service
// implementations themselves live in `ship-primitives::services`; this
// module exists solely to host tests that span both substrate and
// primitives without inviting either crate to take a dev-dependency on
// the other.

#[cfg(test)]
#[path = "tests_provider_adapter.rs"]
mod tests_provider_adapter;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod cross_component_tests;
#[cfg(test)]
#[path = "tests_drain.rs"]
mod tests_drain;
