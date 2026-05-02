/// Compile-time list of SUI island tags — heavy components shipped as
/// separate per-island bundles rather than inlined into sui-components.js.
///
/// Adding a new island: drop the source under
/// `packages/ui-components/src/islands/<tag>/`, add the tag here, rebuild.
/// The daemon route handler and build pipeline pick it up automatically.
pub const SUI_ISLANDS: &[&str] = &["sui-terminal"];
