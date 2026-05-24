//! Rules registry. Each module is one rule.
//!
//! Conventions:
//! - Public `RULE_ID: &str` constant per module
//! - Public `check(path, source) -> Vec<Finding>` per module
//! - Findings carry the `RULE_ID` in the `rule` field
//!
//! Adding a rule = create the module, add the entry to the registry below.

use std::path::Path;
use crate::report::Finding;

pub mod attr_presence;
pub mod controller_context;
pub mod controller_navigation;
pub mod controller_no_ambient_authority;
pub mod controller_purity;
pub mod css_content_text;
pub mod css_literal;
pub mod dimension_literal;
pub mod css_token;
pub mod event_name;
pub mod for_key;
pub mod layout_primitive;
pub mod prop_value;
pub mod space_literal;

/// Function signature every rule conforms to.
pub type RuleFn = fn(&Path, &str) -> Vec<Finding>;

/// Rules that lint a paired `<stem>.controller.ts` source. The dispatcher
/// in `lib.rs::lint_controller_source` walks this slice in order.
pub const CONTROLLER_RULES: &[(&str, RuleFn)] = &[
    (controller_purity::RULE_ID, controller_purity::check),
    (controller_navigation::RULE_ID, controller_navigation::check),
    (controller_context::RULE_ID, controller_context::check),
    (controller_no_ambient_authority::RULE_ID, controller_no_ambient_authority::check),
];

/// Rules that lint a `*.sui` template source. The dispatcher in
/// `lib.rs::lint_sui_template_source` walks this slice in order.
pub const SUI_TEMPLATE_RULES: &[(&str, RuleFn)] = &[
    (css_literal::RULE_ID, css_literal::check),
    (css_content_text::RULE_ID, css_content_text::check),
    (css_token::RULE_ID, css_token::check),
    (for_key::RULE_ID, for_key::check),
    (event_name::RULE_ID, event_name::check),
    (prop_value::RULE_ID, prop_value::check),
    (attr_presence::RULE_ID, attr_presence::check),
    (space_literal::RULE_ID, space_literal::check),
    (layout_primitive::RULE_ID, layout_primitive::check),
    (dimension_literal::RULE_ID, dimension_literal::check),
];
