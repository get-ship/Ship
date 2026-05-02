use std::collections::BTreeSet;

use crate::ir::Instr;

/// Compile-time list of SUI island tags — heavy components shipped as
/// separate per-island bundles rather than inlined into sui-components.js.
///
/// Adding a new island: drop the source under
/// `packages/ui-components/src/islands/<tag>/`, add the tag here, rebuild.
/// The daemon route handler and build pipeline pick it up automatically.
pub const SUI_ISLANDS: &[&str] = &["sui-terminal"];

/// Walk `instrs` recursively and return every distinct island tag encountered,
/// sorted (BTreeSet). Used by the JS emitter and the SSR fragment handler.
pub fn collect_islands(instrs: &[Instr]) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    visit(instrs, &mut found);
    found
}

fn visit(instrs: &[Instr], found: &mut BTreeSet<String>) {
    for instr in instrs {
        match instr {
            Instr::Element(el) => {
                if SUI_ISLANDS.contains(&el.tag.as_str()) {
                    found.insert(el.tag.clone());
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
