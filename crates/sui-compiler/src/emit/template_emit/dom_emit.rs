//! Element + attribute DOM-construction emit.

use super::super::expr_rewrite::rewrite_for_scope;
use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;

impl TemplateEmitter {
    pub(in crate::emit) fn emit_element(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let lower = el.tag.to_ascii_lowercase();
        // Mirror `bindings::Builder::walk_element`: skip `<state>` /
        // `<data>` entirely (no children walked, no keys allocated).
        if lower == "state" || lower == "data" {
            return;
        }
        let v = self.fresh("e");
        w.line(&format!(
            "const {v} = document.createElement({});",
            js_string(&el.tag)
        ));
        for a in &el.attrs {
            self.emit_attr(w, a, &v, cleanups_var);
        }
        self.emit_children(w, &el.children, &v, cleanups_var);
        w.line(&format!("{parent_var}.appendChild({v});"));
    }

    pub(super) fn emit_attr(
        &mut self,
        w: &mut Writer,
        a: &AttrInstr,
        el_var: &str,
        cleanups_var: &str,
    ) {
        match a {
            AttrInstr::Static { name, value, .. } => {
                w.line(&format!(
                    "{el_var}.setAttribute({}, {});",
                    js_string(name),
                    js_string(value)
                ));
            }
            AttrInstr::Expr { name, expr, .. } => {
                let key = self.next_key();
                let rw = rewrite_for_scope(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindAttr({el_var}, {n}, () => ({rw})));",
                    n = js_string(name)
                ));
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateAttr(__m[{k}], {n}, () => ({rw})));",
                        k = js_string(&key),
                        n = js_string(name),
                    ));
                }
            }
            AttrInstr::DynamicAttr { name_expr, expr, .. } => {
                let key = self.next_key();
                self.emit_dynamic_attr(w, name_expr, expr, el_var, cleanups_var);
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.emit_dynamic_attr_hydrate(&key, name_expr, expr);
                }
            }
            AttrInstr::BindValue { expr, .. } => {
                let _ = self.next_key();
                let rw = rewrite_for_scope(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindValue({el_var}, ({rw})));"
                ));
            }
            AttrInstr::BindChecked { expr, .. } => {
                let _ = self.next_key();
                let rw = rewrite_for_scope(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindChecked({el_var}, ({rw})));"
                ));
            }
            AttrInstr::BindThis { var, .. } => {
                let _ = self.next_key();
                // Refs are write-once-from-runtime: create the signal
                // pre-seeded with the freshly constructed element. The
                // component author reads `<var>()` from the controller
                // — same shape as React's `ref.current`, but signal-typed.
                w.line(&format!("$scope.{var} = signal({el_var});"));
            }
            AttrInstr::SlotReflectEmpty { attr, .. } => {
                w.line(&format!(
                    "{cleanups_var}.push(bindSlotEmpty({el_var}, host, {a}));",
                    a = js_string(attr)
                ));
            }
            AttrInstr::On { event, handler, modifiers, .. } => {
                let key = self.next_key();
                let rw = rewrite_for_scope(&handler.source, &[]);
                let mods_js = serialize_modifiers(modifiers);
                w.line(&format!(
                    "{cleanups_var}.push(on({el_var}, {ev}, (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})(), {{ modifiers: {mods_js} }}));",
                    ev = js_string(event)
                ));
                // Phase 3b.4: at depth 0 in views, also emit a direct
                // hydrate call so SSR'd `on:{event}="bN"` markers wire
                // up without going through the runtime walker /
                // `bindings` manifest. The matching `on:*` manifest
                // entry is suppressed in `js.rs::emit_bindings`.
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_markers = true;
                    self.hydrate_calls.line(&format!(
                        "if (__m[{k}]) __cleanups.push(hydrateOn(__m[{k}], {ev}, (function () {{ const __h = ({rw}); return (typeof __h === \"function\") ? __h.bind(undefined) : (() => {{}}); }})(), {mods_js}));",
                        k = js_string(&key),
                        ev = js_string(event),
                    ));
                }
            }
            AttrInstr::Class { name, expr, .. } => {
                let _ = self.next_key();
                let rw = rewrite_for_scope(&expr.source, &[]);
                w.line(&format!(
                    "{cleanups_var}.push(bindClass({el_var}, {n}, () => ({rw})));",
                    n = js_string(name)
                ));
            }
            AttrInstr::Style { name, expr, .. } => {
                let _ = self.next_key();
                let rw = rewrite_for_scope(&expr.source, &[]);
                let prop = js_string(name);
                w.line(&format!(
                    "{cleanups_var}.push(effect(() => {{ const __v = ({rw}); if (__v == null) {el_var}.style.removeProperty({prop}); else {el_var}.style.setProperty({prop}, String(__v)); }}));"
                ));
            }
            AttrInstr::Mutation { expr, .. } => {
                let _ = self.next_key();
                let chain = js_string(&expr.source);
                w.line(&format!(
                    "{cleanups_var}.push(wireFormMutation({el_var}, {chain}, $scope));"
                ));
            }
        }
    }

    fn emit_dynamic_attr(
        &mut self,
        w: &mut Writer,
        name_expr: &Expr,
        value_expr: &Expr,
        el_var: &str,
        cleanups_var: &str,
    ) {
        let n_rw = rewrite_for_scope(&name_expr.source, &[]);
        let v_rw = rewrite_for_scope(&value_expr.source, &[]);
        let prev = self.fresh("prevAttr");
        w.line(&format!("{{ let {prev} = null;"));
        w.line(&format!("  {cleanups_var}.push(effect(() => {{"));
        w.line(&format!("    const __n = String({n_rw});"));
        w.line(&format!("    const __v = ({v_rw});"));
        w.line(&format!(
            "    if ({prev} && {prev} !== __n) {el_var}.removeAttribute({prev});"
        ));
        w.line(&format!("    {prev} = __n;"));
        w.line(&format!(
            "    if (__v == null || __v === false || __v === \"\") {el_var}.removeAttribute(__n);"
        ));
        w.line(&format!(
            "    else if (__v === true) {el_var}.setAttribute(__n, \"\");"
        ));
        w.line(&format!("    else {el_var}.setAttribute(__n, String(__v));"));
        w.line("  })); }");
    }

    /// Hydrate-side mirror of `emit_dynamic_attr`. The marker carrying
    /// the binding key is `attr-dynamic="bN"` on the SSR'd element, so
    /// `__m[bN]` resolves to the same element the CSR effect would
    /// observe. The body matches CSR exactly, modulo the element source.
    fn emit_dynamic_attr_hydrate(&mut self, key: &str, name_expr: &Expr, value_expr: &Expr) {
        let n_rw = rewrite_for_scope(&name_expr.source, &[]);
        let v_rw = rewrite_for_scope(&value_expr.source, &[]);
        let k = js_string(key);
        self.hydrate_calls.line(&format!("if (__m[{k}]) {{"));
        self.hydrate_calls.line(&format!("  const __el = __m[{k}];"));
        self.hydrate_calls.line("  let __prev = null;");
        self.hydrate_calls.line("  __cleanups.push(effect(() => {");
        self.hydrate_calls.line(&format!("    const __n = String({n_rw});"));
        self.hydrate_calls.line(&format!("    const __v = ({v_rw});"));
        self.hydrate_calls.line("    if (__prev && __prev !== __n) __el.removeAttribute(__prev);");
        self.hydrate_calls.line("    __prev = __n;");
        self.hydrate_calls.line(
            "    if (__v == null || __v === false || __v === \"\") __el.removeAttribute(__n);",
        );
        self.hydrate_calls.line("    else if (__v === true) __el.setAttribute(__n, \"\");");
        self.hydrate_calls.line("    else __el.setAttribute(__n, String(__v));");
        self.hydrate_calls.line("  }));");
        self.hydrate_calls.line("}");
    }
}

pub(super) fn serialize_modifiers(mods: &[Modifier]) -> String {
    if mods.is_empty() {
        return "[]".into();
    }
    let parts: Vec<String> = mods.iter().map(modifier_js).collect();
    format!("[{}]", parts.join(", "))
}

fn modifier_js(m: &Modifier) -> String {
    match m {
        Modifier::PreventDefault => "{ kind: \"preventDefault\" }".into(),
        Modifier::StopPropagation => "{ kind: \"stopPropagation\" }".into(),
        Modifier::StopImmediatePropagation => {
            "{ kind: \"stopImmediatePropagation\" }".into()
        }
        Modifier::SelfMod => "{ kind: \"self\" }".into(),
        Modifier::Trusted => "{ kind: \"trusted\" }".into(),
        Modifier::Key(arg) => with_arg("key", arg.as_deref()),
        Modifier::Debounce(arg) => with_arg("debounce", arg.as_deref()),
        Modifier::Throttle(arg) => with_arg("throttle", arg.as_deref()),
    }
}

fn with_arg(kind: &str, arg: Option<&str>) -> String {
    match arg {
        Some(a) => format!("{{ kind: \"{kind}\", arg: {} }}", js_string(a)),
        None => format!("{{ kind: \"{kind}\" }}"),
    }
}
