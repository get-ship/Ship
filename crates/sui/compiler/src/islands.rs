use std::collections::BTreeSet;

use crate::ir::Instr;

/// Compile-time map of `<sui-*>` separate-bundle tags to their bundle name
/// — each bundle ships as a separate `/_sui/sui-components/<bundle>.js`
/// file rather than inlined into `sui-components.js`.
///
/// Most tags are their own bundle (`tag == bundle`). Family bundles
/// pack multiple tags that share controllers/styles into a single
/// asset (e.g. the command-bar family); the tag→bundle column lets the
/// daemon serve one file regardless of which family member appears in
/// a template.
///
/// Adding a new separate-bundle component: drop the source under
/// `packages/ship-components/ui-components/src/<bundle>/`, mark its `.sui` source with a
/// top-of-file `<bundle separate />` directive, list its tag here, and
/// rebuild. The daemon route handler and build pipeline pick it up
/// automatically.
pub const SUI_ISLANDS: &[(&str, &str)] = &[
    ("sui-terminal", "sui-terminal"),
    ("sui-command-bar", "sui-command-bar"),
    ("sui-command-palette", "sui-command-bar"),
    ("sui-command-chat", "sui-command-bar"),
    ("sui-devtools-source", "sui-devtools"),
    ("sui-devtools-components", "sui-devtools"),
    ("sui-devtools-lint", "sui-devtools"),
    ("sui-devtools-schema", "sui-devtools"),
    ("sui-devtools-signals", "sui-devtools"),
    ("sui-launcher", "sui-launcher"),
    ("sui-panel-dock", "sui-panel-dock"),
    ("sui-nav-top", "sui-nav-top"),
    ("sui-markdown-editor", "sui-markdown-editor"),
    ("sui-code-editor", "sui-code-editor"),
];

pub fn island_bundle(tag: &str) -> Option<&'static str> {
    SUI_ISLANDS.iter().find(|(t, _)| *t == tag).map(|(_, b)| *b)
}

pub fn is_island_bundle(name: &str) -> bool {
    SUI_ISLANDS.iter().any(|(_, b)| *b == name)
}

/// Walk `instrs` recursively and return every distinct island BUNDLE
/// the template needs (deduped — multiple tags from the same family
/// resolve to one entry). Used by the JS emitter and the SSR fragment
/// handler so the host can fetch each bundle exactly once.
pub fn collect_islands(instrs: &[Instr]) -> BTreeSet<&'static str> {
    let mut found = BTreeSet::new();
    visit(instrs, &mut found);
    found
}

fn visit(instrs: &[Instr], found: &mut BTreeSet<&'static str>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if let Some(bundle) = island_bundle(el.tag.as_str()) {
                    found.insert(bundle);
                }
                visit(&el.children, found);
            }
            Instr::If(i) => {
                for branch in &i.branches {
                    visit(&branch.body, found);
                }
                if let Some(else_body) = &i.else_body {
                    visit(else_body, found);
                }
            }
            Instr::For(f) => {
                visit(&f.body, found);
                if let Some(else_body) = &f.else_body {
                    visit(else_body, found);
                }
            }
            Instr::Match(m) => {
                for case in &m.cases {
                    visit(&case.body, found);
                }
                if let Some(default) = &m.default {
                    visit(default, found);
                }
            }
            Instr::WhileLoading(wl) => visit(&wl.body, found),
            Instr::WhileError(we) => visit(&we.body, found),
            Instr::StaticText { .. } | Instr::DynamicText { .. } | Instr::Comment { .. } => {}
        }
    }
}
