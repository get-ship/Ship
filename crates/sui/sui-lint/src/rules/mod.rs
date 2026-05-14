//! Rules registry. Each module is one rule.
//!
//! Conventions:
//! - Public `RULE_ID: &str` constant per module
//! - Public `check(path, source) -> Vec<Finding>` per module
//! - Findings carry the `RULE_ID` in the `rule` field
//!
//! Adding a rule = create the module, add the entry to the registry below.

use std::path::Path;
use lint::report::Finding;

pub mod controller_purity;
pub mod css_token;
pub mod event_name;
pub mod for_key;

/// Function signature every rule conforms to.
pub type RuleFn = fn(&Path, &str) -> Vec<Finding>;

/// Rules that lint a paired `<stem>.controller.ts` source. The dispatcher
/// in `lib.rs::lint_controller_source` walks this slice in order.
pub const CONTROLLER_RULES: &[(&str, RuleFn)] = &[
    (controller_purity::RULE_ID, controller_purity::check),
];

/// Rules that lint a `*.sui` template source. The dispatcher in
/// `lib.rs::lint_sui_template_source` walks this slice in order.
pub const SUI_TEMPLATE_RULES: &[(&str, RuleFn)] = &[
    (css_token::RULE_ID, css_token::check),
    (for_key::RULE_ID, for_key::check),
    (event_name::RULE_ID, event_name::check),
];
