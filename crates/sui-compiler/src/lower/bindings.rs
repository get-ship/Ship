//! Per-attribute import collection. Mirrors the runtime helpers
//! exposed in `packages/ui-runtime/src/index.ts`.

use crate::ir::*;
use crate::lower::Lowered;

pub fn collect_attr_imports(a: &AttrInstr, out: &mut Lowered) {
    match a {
        AttrInstr::Static { .. } => {}
        AttrInstr::Expr { .. } => {
            out.runtime_imports.insert("bindAttr");
        }
        AttrInstr::DynamicAttr { .. } => {
            out.runtime_imports.insert("effect");
        }
        AttrInstr::BindValue { .. } => {
            out.runtime_imports.insert("bindValue");
        }
        AttrInstr::BindChecked { .. } => {
            out.runtime_imports.insert("bindChecked");
        }
        AttrInstr::On { .. } => {
            out.runtime_imports.insert("on");
        }
        AttrInstr::Class { .. } => {
            out.runtime_imports.insert("bindClass");
        }
        AttrInstr::Style { .. } => {
            out.runtime_imports.insert("effect");
        }
        AttrInstr::Mutation { .. } => {
            out.runtime_imports.insert("wireFormMutation");
        }
    }
}
