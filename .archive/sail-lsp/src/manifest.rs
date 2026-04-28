//! Static manifests embedded into the LSP binary at build time.
//!
//! Two sources of truth feed completion + hover:
//!
//! - `sdk-framework.json` — directives, scope bindings, lifecycle
//!   hooks, template syntax forms, reactivity rules. Reused via
//!   [`sail_compiler::Manifest`] (the JSON shape is a superset; serde
//!   ignores `_generated` and `$schema`).
//! - `sdk-ship-ops.json` — every `ship.<domain>.<op>` with description,
//!   args, returns, and permission.
//!
//! Both are bundled with `include_str!` and parsed lazily through
//! `OnceLock` so the cost is paid once per process and shared by every
//! request handler.
//!
//! Failure mode: a malformed embedded manifest is a build-broken
//! product, not a runtime user condition. The accessors `panic!` with
//! the parse error so misshapen manifests fail tests loudly.

use std::sync::OnceLock;

use sail_compiler::Manifest as FrameworkManifest;
use serde::Deserialize;

const FRAMEWORK_JSON: &str =
    include_str!("../../../packages/ship-set/sdk/sdk-framework.json");
const SHIP_OPS_JSON: &str =
    include_str!("../../../packages/ship-set/sdk/sdk-ship-ops.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ShipOpsManifest {
    pub entries: std::collections::BTreeMap<String, ShipOp>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShipOp {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub args: Vec<ShipOpArg>,
    pub returns: String,
    pub permission: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShipOpArg {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub required: bool,
}

static FRAMEWORK: OnceLock<FrameworkManifest> = OnceLock::new();
static SHIP_OPS: OnceLock<ShipOpsManifest> = OnceLock::new();

pub fn framework() -> &'static FrameworkManifest {
    FRAMEWORK.get_or_init(|| {
        serde_json::from_str(FRAMEWORK_JSON)
            .expect("embedded sdk-framework.json must parse")
    })
}

pub fn ship_ops() -> &'static ShipOpsManifest {
    SHIP_OPS.get_or_init(|| {
        serde_json::from_str(SHIP_OPS_JSON)
            .expect("embedded sdk-ship-ops.json must parse")
    })
}

impl ShipOp {
    /// One-line summary suitable for `CompletionItem::detail`.
    pub fn signature(&self) -> String {
        let args: Vec<String> = self
            .args
            .iter()
            .map(|a| {
                if a.required {
                    format!("{}: {}", a.name, a.kind)
                } else {
                    format!("{}?: {}", a.name, a.kind)
                }
            })
            .collect();
        format!("ship.{}({}) -> {}", self.name, args.join(", "), self.returns)
    }

    /// Markdown-formatted documentation block.
    pub fn markdown(&self) -> String {
        let mut s = String::new();
        s.push_str("```ts\n");
        s.push_str(&self.signature());
        s.push_str("\n```\n\n");
        s.push_str(&self.description);
        s.push_str("\n\nrequires `");
        s.push_str(&self.permission);
        s.push('`');
        s
    }

    pub fn domain(&self) -> &str {
        self.name.split_once('.').map(|(d, _)| d).unwrap_or(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifests_load_lazily_and_parse() {
        let fw = framework();
        assert!(!fw.directives.is_empty(), "directives missing");
        assert!(!fw.scope_bindings.is_empty(), "scope bindings missing");
        assert!(!fw.lifecycle_hooks.is_empty(), "lifecycle hooks missing");
        assert!(!fw.template_syntax.is_empty(), "template syntax missing");

        let ops = ship_ops();
        assert!(!ops.entries.is_empty(), "ship ops missing");
        // OnceLock identity holds across calls.
        assert!(std::ptr::eq(framework(), fw));
        assert!(std::ptr::eq(ship_ops(), ops));
    }

    #[test]
    fn ship_op_signature_and_markdown_render() {
        let ops = ship_ops();
        let detail = ops
            .entries
            .get("agents.detail")
            .expect("agents.detail present");
        let sig = detail.signature();
        assert!(sig.starts_with("ship.agents.detail("), "{sig}");
        assert!(sig.contains("id: string"), "{sig}");
        let md = detail.markdown();
        assert!(md.contains("```ts"));
        assert!(md.contains(&detail.description));
        assert!(md.contains(&detail.permission));
        assert_eq!(detail.domain(), "agents");
    }
}
