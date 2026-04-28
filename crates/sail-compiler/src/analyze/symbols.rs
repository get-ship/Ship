//! Local symbol table for a parsed `.set` file. Names → what kind of
//! symbol they are (signal / function / event), with the type info we
//! captured at parse time. Built from the AST without external state —
//! every reactivity rule (R1, R2, R5) and completion provider walks
//! this rather than re-implementing AST traversal.

use std::collections::BTreeMap;

use crate::parse::ast::{ScriptBlock, SailFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalKind {
    /// `prop X[: T][!] [= default]` — public reactive property.
    Prop { ty: Option<String>, required: bool },
    /// `state X[: T] = initial` — local signal seeded at mount.
    State { ty: Option<String> },
    /// `derived X = expr` — memoised computed signal.
    Derived,
    /// `data X[: T] = ship.<domain>.<op>(args)` — server-truth signal.
    Data { ty: Option<String> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolTable {
    /// Every reactive name in scope, mapped to its kind + declared type.
    /// Ordered for stable diagnostic output.
    pub signals: BTreeMap<String, SignalKind>,
    /// Plain JS functions declared at script top-level. Not signals;
    /// callable as `name(args)` in any handler/effect/mount.
    pub functions: BTreeMap<String, FunctionInfo>,
    /// `event NAME[: T]` declarations.
    pub events: BTreeMap<String, EventInfo>,
    /// `component formAssociated` flag.
    pub form_associated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionInfo {
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInfo {
    pub payload_ty: Option<String>,
}

impl SymbolTable {
    /// Build the table from a parsed file. Returns an empty table when
    /// the file has no script block (template-only / style-only files).
    pub fn from_file(file: &SailFile) -> Self {
        let Some(script) = file.script.as_ref() else {
            return Self::default();
        };
        Self::from_script(script)
    }

    pub fn from_script(script: &ScriptBlock) -> Self {
        let mut signals: BTreeMap<String, SignalKind> = BTreeMap::new();
        for p in &script.props {
            signals.insert(p.name.clone(), SignalKind::Prop {
                ty: p.ty.clone(),
                required: p.required,
            });
        }
        for s in &script.state {
            signals.insert(s.name.clone(), SignalKind::State { ty: s.ty.clone() });
        }
        for d in &script.derived {
            signals.insert(d.name.clone(), SignalKind::Derived);
        }
        for d in &script.data {
            signals.insert(d.name.clone(), SignalKind::Data { ty: d.ty.clone() });
        }
        let functions = script.functions.iter()
            .map(|f| (f.name.clone(), FunctionInfo { raw: f.raw.clone() }))
            .collect();
        let events = script.events.iter()
            .map(|e| (e.name.clone(), EventInfo { payload_ty: e.payload_ty.clone() }))
            .collect();
        Self {
            signals,
            functions,
            events,
            form_associated: script.form_associated,
        }
    }

    /// Is the given identifier a reactive signal in this file?
    pub fn is_signal(&self, name: &str) -> bool {
        self.signals.contains_key(name)
    }

    /// Is the given identifier a top-level function?
    pub fn is_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Total count of names this table owns. Useful for sanity tests.
    pub fn len(&self) -> usize {
        self.signals.len() + self.functions.len() + self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn build(src: &str) -> SymbolTable {
        let file = parse::parse(src).expect("parse");
        SymbolTable::from_file(&file)
    }

    #[test]
    fn empty_file_yields_empty_table() {
        let t = build("<template>x</template>");
        assert!(t.is_empty());
        assert!(!t.is_signal("foo"));
    }

    #[test]
    fn captures_props_state_derived_data() {
        let t = build("\
<script>
prop label: string = ''
state count = 0
derived doubled = count() * 2
data agents = ship.agents.summaries()
</script>
<template>x</template>");
        assert!(matches!(t.signals.get("label"), Some(SignalKind::Prop { .. })));
        assert!(matches!(t.signals.get("count"), Some(SignalKind::State { .. })));
        assert!(matches!(t.signals.get("doubled"), Some(SignalKind::Derived)));
        assert!(matches!(t.signals.get("agents"), Some(SignalKind::Data { .. })));
        assert!(t.is_signal("doubled"));
        assert!(!t.is_signal("missing"));
    }

    #[test]
    fn captures_functions_and_events() {
        let t = build("\
<script>
function close() { open.set(false) }
event submit: { value: string }
</script>
<template>x</template>");
        assert!(t.is_function("close"));
        assert!(!t.is_signal("close"));
        let evt = t.events.get("submit").expect("event present");
        assert_eq!(evt.payload_ty.as_deref(), Some("{ value: string }"));
    }

    #[test]
    fn captures_form_associated_flag() {
        let t = build("\
<script>
component formAssociated
prop value: string = ''
</script>
<template>x</template>");
        assert!(t.form_associated);
    }

    #[test]
    fn prop_required_marker_round_trips() {
        let t = build("\
<script>
prop count: number! = 0
</script>
<template>x</template>");
        match t.signals.get("count") {
            Some(SignalKind::Prop { required, .. }) => assert!(*required),
            other => panic!("expected required prop, got {other:?}"),
        }
    }
}
