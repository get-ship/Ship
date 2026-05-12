//! Control-flow emit: `{#if}`, `{#for}`, `{#match}`, `{#while-loading}`,
//! and `{#while-error}`.
//!
//! Phase 3b.5 (gate H): control-flow boundaries are direct-emit through
//! the unified `hydrate(root, ctx)` function on the view's setup return.
//! Each `{#if}` / `{#for}` / `{#match}` boundary still hoists a
//! `__wireBoundary_b<N>` const inside `setup(ctx)` so the legacy
//! `__mount` path can call it inline; the hydrate function looks up the
//! SSR'd `<sui-{kind} key="b<N>">` element via `collectElementMarkers`,
//! swaps it for a Comment anchor, and invokes the same hoisted helper.
//! `__b<N>_attach` named properties are no longer emitted — `attach()`
//! drives boundaries through hydrate now, not through `wireBoundaries`.
//!
//! `{#while-loading}` / `{#while-error}` follow the same shape via
//! `hydrateWhileLoading` / `hydrateWhileError` (gate G).

use super::super::expr_rewrite::rewrite_for_scope;
use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;

impl TemplateEmitter {
    pub(in crate::emit) fn emit_if(
        &mut self,
        w: &mut Writer,
        instr: &IfInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("ifAnchor");

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"if\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        // Direct-emit hydrate dispatch (parent before children — outer
        // boundary swaps detach inner SSR'd nodes, so inner hydrate
        // calls become no-ops via the `__m[k]` guard).
        if self.is_view && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "if");
        }

        // Hoisted handler — emitted into `boundary_hoists`, drained
        // into the setup body before the `return`.
        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindIf({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        for (i, b) in instr.branches.iter().enumerate() {
            let rw = rewrite_for_scope(&b.cond.source, &[]);
            h.line(&format!("if ({rw}) return {i};"));
        }
        let fallback = if instr.else_body.is_some() {
            instr.branches.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for b in &instr.branches {
            self.emit_branch(&mut h, &b.body);
        }
        if let Some(eb) = &instr.else_body {
            self.emit_branch(&mut h, eb);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    pub(in crate::emit) fn emit_for(
        &mut self,
        w: &mut Writer,
        instr: &ForInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("forAnchor");

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"for\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        if self.is_view && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "for");
        }

        let it_rw = rewrite_for_scope(&instr.iterable.source, &[]);
        let var = &instr.var_name;
        let idx_name = instr.index_name.as_deref().unwrap_or("__i");

        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindFor({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line(&format!("items: () => ({it_rw}),"));
        if let Some(k) = &instr.key {
            let k_rw = rewrite_for_scope(&k.source, &[var.as_str(), idx_name]);
            h.line(&format!(
                "key: ({var}, {idx_name}) => {{ const __child = Object.create($scope); __child.{var} = {var}; __child.{idx_name} = {idx_name}; return (($scope) => ({k_rw}))(__child); }},"
            ));
        }
        h.line(&format!("render: ({var}, {idx_name}) => {{"));
        h.enter();
        let child_scope = self.fresh("childScope");
        h.line(&format!("const {child_scope} = Object.create($scope);"));
        h.line(&format!("{child_scope}.{var} = {var};"));
        h.line(&format!("{child_scope}.{idx_name} = {idx_name};"));
        // Per-row version signal. Incremented by `update` to force all
        // reactive binding closures inside this row to re-evaluate,
        // picking up the new item value from $scope.{var}.
        h.line(&format!("{child_scope}.__v = signal(0);"));
        // Wrap the body in an IIFE that takes `$scope` as a parameter so
        // the iteration's child scope shadows the outer `$scope` without
        // a same-block `const $scope` declaration (which would TDZ on
        // the `Object.create($scope)` above).
        h.line("return (($scope) => {");
        h.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        // Walk body into a separate writer so we can inject any nested
        // boundary hoists (from {#if}/{#for}/{#match} inside the for
        // body) BEFORE the body consumes them. Without this, nested
        // hoists would land in `boundary_hoists` and emit at top-level —
        // closing over the outer `$scope` instead of the iteration's
        // IIFE-bound one. References like `$scope.item` would then
        // resolve to undefined inside nested test/render closures.
        let mut body_w = Writer::new();
        body_w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        body_w.line(&format!("const {cl} = [];"));
        let hoist_snapshot = self.boundary_hoists.len();
        self.boundary_depth += 1;
        let outer_for = std::mem::replace(&mut self.for_in_scope, true);
        self.emit_children(&mut body_w, &instr.body, &frag, &cl);
        self.for_in_scope = outer_for;
        self.boundary_depth -= 1;
        let captured_hoists = self.boundary_hoists.split_off_from(hoist_snapshot);
        if !captured_hoists.is_empty() {
            // Captured hoist text was generated at indent=0; placing it
            // inside the IIFE leaves it left-aligned (ugly but valid JS).
            h.push_str(&captured_hoists);
        }
        h.push_str(&body_w.into_string());
        let new_var = self.fresh("new");
        let new_idx = self.fresh("newI");
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }}, update: ({new_var}, {new_idx}) => {{ $scope.{var} = {new_var}; $scope.{idx_name} = {new_idx}; $scope.__v.set(c => c + 1); }} }};"
        ));
        h.leave();
        h.line(&format!("}})({child_scope});"));
        h.leave();
        h.line("},");
        if let Some(eb) = &instr.else_body {
            h.line("emptyRender: () => {");
            h.enter();
            let frag = self.fresh("frag");
            let cl = self.fresh("c");
            h.line(&format!(
                "const {frag} = document.createDocumentFragment();"
            ));
            h.line(&format!("const {cl} = [];"));
            self.boundary_depth += 1;
            self.emit_children(&mut h, eb, &frag, &cl);
            self.boundary_depth -= 1;
            h.line(&format!(
                "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
            ));
            h.leave();
            h.line("},");
        }
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    pub(in crate::emit) fn emit_match(
        &mut self,
        w: &mut Writer,
        instr: &MatchInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("matchAnchor");

        w.line(&format!(
            "const {anchor} = document.createComment(\"match\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        if self.is_view && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.emit_boundary_hydrate(&key, "match");
        }

        let disc_rw = rewrite_for_scope(&instr.discriminant.source, &[]);
        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindIf({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        h.line(&format!("const __d = ({disc_rw});"));
        for (i, c) in instr.cases.iter().enumerate() {
            let c_rw = rewrite_for_scope(&c.test.source, &[]);
            h.line(&format!(
                "if (__d === ({c_rw})) return {i};"
            ));
        }
        let fallback = if instr.default.is_some() {
            instr.cases.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for c in &instr.cases {
            self.emit_branch(&mut h, &c.body);
        }
        if let Some(d) = &instr.default {
            self.emit_branch(&mut h, d);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());
    }

    /// Emit a hydrate-side dispatch for an `if` / `for` / `match`
    /// boundary. The compiled `hydrate(root, ctx)` body looks up the
    /// SSR'd `<sui-{kind} key="b<N>">` element via `__m`, swaps it for
    /// a Comment anchor, and invokes the hoisted `__wireBoundary_b<N>`
    /// helper — same shape the legacy `__bN_attach` named property used
    /// to expose, just inlined into the unified hydrate function.
    fn emit_boundary_hydrate(&mut self, key: &str, kind: &str) {
        let k = js_string(key);
        self.hydrate_calls.line(&format!("if (__m[{k}]) {{"));
        self.hydrate_calls.line(&format!("  const __t = __m[{k}];"));
        self.hydrate_calls.line("  const __doc = __t.ownerDocument ?? document;");
        self.hydrate_calls.line(&format!(
            "  const __anchor = (__t.nodeType === 1) ? __doc.createComment({}) : __t;",
            js_string(kind),
        ));
        self.hydrate_calls.line(
            "  if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);",
        );
        self.hydrate_calls.line(&format!(
            "  __cleanups.push(__wireBoundary_{key}(__anchor));"
        ));
        self.hydrate_calls.line("}");
    }

    /// Emit `{#while-loading of <bind>}…{/while-loading}`.
    ///
    /// Hoists `const __renderWhileLoading_<N> = () => …` inside
    /// `setup(ctx)` so the body is walked exactly once. Both `__mount`
    /// (legacy) and `hydrate(root, ctx)` reference the same hoisted const.
    /// Single walk = single registry-key consumption per body.
    pub(super) fn emit_while_loading_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileLoadingInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        // Registry parity: while-loading consumes one b<N> for the
        // boundary itself. Body keys are consumed during the (single)
        // emit walk below.
        let key = self.next_key();
        let idx = self.while_loading_idx;
        self.while_loading_idx += 1;
        let render_key = format!("__renderWhileLoading_{idx}");
        let bind = &instr.bind;

        // Hoist the render fn so it's a single-source-of-truth.
        self.emit_wl_render_hoist(&render_key, &instr.body);

        // __mount inline call references the hoisted const.
        let anchor = self.fresh("wlAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-loading\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileLoading({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        // Phase 3b.5 gate G: direct-emit hydrate call. The render fn is
        // already in the setup closure (no need to expose it as a named
        // property anymore).
        if self.is_view && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.hydrate_calls.line(&format!(
                "if (__m[{k}]) __cleanups.push(hydrateWhileLoading(__m[{k}], {k}, () => $scope.{bind}(), {render_key}));",
                k = js_string(&key),
            ));
        }
    }

    /// Emit `{#while-error <errVar> of <bind>}…{/while-error}`.
    pub(super) fn emit_while_error_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileErrorInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let idx = self.while_error_idx;
        self.while_error_idx += 1;
        let render_key = format!("__renderWhileError_{idx}");
        let bind = &instr.bind;
        let var = &instr.error_var;

        self.emit_we_render_hoist(&render_key, var, &instr.body);

        let anchor = self.fresh("weAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-error\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileError({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        if self.is_view && self.boundary_depth == 0 {
            self.hydrate_uses_markers = true;
            self.hydrate_calls.line(&format!(
                "if (__m[{k}]) __cleanups.push(hydrateWhileError(__m[{k}], {k}, () => $scope.{bind}(), {render_key}));",
                k = js_string(&key),
            ));
        }
    }

    fn emit_wl_render_hoist(&mut self, render_key: &str, body: &[Instr]) {
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let mut h = Writer::new();
        h.line(&format!("const {render_key} = () => {{"));
        h.enter();
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&format!("const {cl} = [];"));
        self.boundary_depth += 1;
        self.emit_children(&mut h, body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }

    fn emit_we_render_hoist(&mut self, render_key: &str, error_var: &str, body: &[Instr]) {
        let child = self.fresh("childScope");
        let var = error_var;
        let mut h = Writer::new();
        h.line(&format!("const {render_key} = ({var}) => {{"));
        h.enter();
        h.line(&format!("const {child} = Object.create($scope);"));
        h.line(&format!("{child}.{var} = {var};"));
        // IIFE-shadow `$scope` to avoid the TDZ that would fire if we
        // declared `const $scope = {child}` in the same block as the
        // `Object.create($scope)` above.
        h.line("return (($scope) => {");
        h.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&format!("const {cl} = [];"));
        self.boundary_depth += 1;
        self.emit_children(&mut h, body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line(&format!("}})({child});"));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }
}
