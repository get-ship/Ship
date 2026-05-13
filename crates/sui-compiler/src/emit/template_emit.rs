//! Walk template `Instr`s and emit DOM-construction JS into a writer.
//!
//! Each instruction maps to a small chunk of JS that creates DOM
//! nodes, registers reactive bindings, and pushes their disposers
//! onto a `__cleanups` array.
//!
//! Expressions are pasted inline. The compiler rewrites each
//! template-expression source so free identifiers are qualified with
//! `$scope.` (see `expr_rewrite`), then the rewritten string is
//! emitted directly into a closure body — no `with($scope)`, no
//! `new Function` eval, no shared expression bank.
//!
//! ## Phase 2 — client-side control flow
//!
//! `{#if}` / `{#for}` / `{#match}` boundaries no longer round-trip
//! through `/_sui/render-fragment/`. Each boundary emits two pieces:
//!
//!   1. A hoisted `const __wireBoundary_b<N> = (anchor) => bindIf({...})`
//!      defined inside `setup(ctx)`. Captures `$scope`. Used by
//!      `__mount`'s inline path AND by the public attach handler below.
//!   2. (Views only) A public property `__b<N>_attach: (target) => …`
//!      on the setup return. `attach()` walks SSR'd
//!      `<sui-{if|for|match|await} key="b<N>">` elements, finds the
//!      handler by key, swaps the SSR wrapper for a Comment anchor, and
//!      invokes the hoisted helper. The first effect-run mounts the
//!      active branch synchronously, replacing the SSR'd content.
//!
//! The emit logic is split across three modules:
//!  * this file — orchestration + the `emit_one` dispatcher
//!  * `dom_emit` — element + attribute emit
//!  * `control_emit` — `{#if}`, `{#for}`, `{#match}`, `{#while-*}`

use super::expr_rewrite::rewrite_for_scope;
use super::writer::{js_string, Writer};
use crate::ir::*;
use crate::keys::{BindingRegistry, KeyAllocator};

mod control_emit;
mod dom_emit;

pub struct TemplateEmitter {
    next_id: usize,
    /// Sequential counters for while-loading/while-error render-callback names.
    /// Both the bindings pass and this emit pass walk the IR in the same source
    /// order, so the N-th while-loading gets the same `__renderWhileLoading_<N>`
    /// name in both.
    pub(super) while_loading_idx: usize,
    pub(super) while_error_idx: usize,
    /// Render callbacks for `{#while-loading}` / `{#while-error}` blocks,
    /// emitted as named properties on the setup return object (outside
    /// `__mount`) so `attach()` can look them up by `renderKey`.
    pub(super) render_callbacks: Writer,
    /// Hoisted `const __wireBoundary_b<N> = (anchor) => bindIf(...)`
    /// declarations for each `{#if}` / `{#for}` / `{#match}` boundary.
    /// Drained into the setup body before the return statement so the
    /// helpers are in scope for both `__mount` and the hydrate-side
    /// boundary dispatch emitted into `hydrate_calls`.
    pub(super) boundary_hoists: Writer,
    /// Direct-emit hydrate calls for migrated binding kinds (Phase 3b.3).
    /// Each line is one statement inside the view's `hydrate(root, ctx)`
    /// function — e.g. `if (__t["b0"]) __cleanups.push(hydrateText(...));`.
    /// Only populated in view mode; components stay manifest-driven this
    /// phase. The corresponding manifest entries are skipped in
    /// `js.rs::emit_bindings`, so each migrated kind moves out of the
    /// manifest and into hydrate atomically.
    pub(super) hydrate_calls: Writer,
    /// True iff `hydrate_calls` includes any text-binding wiring — drives
    /// the `collectTextOpens(root)` preamble emit in `js.rs`.
    pub(super) hydrate_uses_text: bool,
    /// True iff `hydrate_calls` includes any element-binding wiring
    /// (`on:*`, `attr:*`, `class:*`, `style:*`, `bind:value`, `bind:this`)
    /// — drives the `collectElementMarkers(root)` preamble emit in
    /// `js.rs`.
    pub(super) hydrate_uses_markers: bool,
    /// Depth counter for emits inside `{#if}` / `{#for}` / `{#match}` /
    /// `{#while-*}` bodies. Hydrate calls fire only at depth 0 — bindings
    /// inside boundary subtrees are wired by the boundary's own render
    /// closure (which builds DOM client-side and calls `bindText` etc.
    /// directly). The SSR'd markers inside boundary subtrees are detached
    /// when `wireBoundaries` swaps the wrapper, so `collectTextOpens`
    /// would not see them anyway.
    pub(super) boundary_depth: usize,
    /// Key allocator that mirrors `bindings::Builder` exactly. Bumped at
    /// every IR position the registry pre-pass would have allocated a
    /// key — so the `b<N>` we read here for control-flow boundaries
    /// matches the SSR walker's emitted `<sui-{kind} key="b<N>">`.
    pub(super) keys: KeyAllocator,
    /// Whether the current compile mode is `View`. Components don't
    /// emit `hydrate(root, ctx)` (no SSR/attach() path).
    pub(super) is_view: bool,
    /// True while emitting the per-item IIFE body of a `{#for}` loop.
    /// When set, reactive binding closures void-read `$scope.__v` to
    /// track the per-row version signal and re-run when `update` fires.
    pub(super) for_in_scope: bool,
    /// D5: statements that re-execute an inlined static assignment
    /// (`__t.textContent = $scope.row.label;`, …). Accumulated while
    /// emitting the current `{#for}` row body; spliced into that for's
    /// `update()` callback so keyed-row reuse refreshes the inlined
    /// DOM. `emit_for` saves/restores this list around its body walk
    /// to scope it per for-loop.
    pub(super) inline_update_stmts: Vec<String>,
}

impl TemplateEmitter {
    pub fn new(_registry: &BindingRegistry, is_view: bool) -> Self {
        Self {
            next_id: 0,
            while_loading_idx: 0,
            while_error_idx: 0,
            render_callbacks: Writer::new(),
            boundary_hoists: Writer::new(),
            hydrate_calls: Writer::new(),
            hydrate_uses_text: false,
            hydrate_uses_markers: false,
            boundary_depth: 0,
            keys: KeyAllocator::default(),
            is_view,
            for_in_scope: false,
            inline_update_stmts: Vec::new(),
        }
    }

    /// Drain the render-callback buffer. Called by `emit_setup_body`
    /// after `emit_top_level` finishes writing `__mount`.
    pub fn take_render_callbacks(&mut self) -> String {
        std::mem::replace(&mut self.render_callbacks, Writer::new()).into_string()
    }

    /// Drain the hoisted-handler buffer. Called by `emit_setup_body`
    /// before the `return` statement so the consts are in scope.
    pub fn take_boundary_hoists(&mut self) -> String {
        std::mem::replace(&mut self.boundary_hoists, Writer::new()).into_string()
    }

    /// Drain the hydrate-call buffer. Returns the body of the view's
    /// `hydrate(root, ctx)` named property — empty when no leaf binding
    /// kinds have been migrated to direct-emit yet.
    pub fn take_hydrate_calls(&mut self) -> String {
        std::mem::replace(&mut self.hydrate_calls, Writer::new()).into_string()
    }

    /// Whether `take_hydrate_calls` will need the `collectTextOpens(root)`
    /// preamble (i.e. at least one text binding migrated).
    pub fn hydrate_uses_text(&self) -> bool {
        self.hydrate_uses_text
    }

    /// Whether `take_hydrate_calls` will need the
    /// `collectElementMarkers(root)` preamble (i.e. at least one
    /// element-bound leaf kind migrated).
    pub fn hydrate_uses_markers(&self) -> bool {
        self.hydrate_uses_markers
    }

    /// Returns `"void $scope.__v(), "` when inside a `{#for}` IIFE body,
    /// so reactive getter closures track the per-row version signal.
    /// Used as a prefix inside `() => (…)` style closures.
    pub(super) fn for_version_comma(&self) -> &'static str {
        if self.for_in_scope { "void $scope.__v(), " } else { "" }
    }

    /// Returns `"void $scope.__v(); "` when inside a `{#for}` IIFE body.
    /// Used as a leading statement inside `effect(() => { … })` bodies.
    pub(super) fn for_version_stmt(&self) -> &'static str {
        if self.for_in_scope { "void $scope.__v(); " } else { "" }
    }

    /// D5: classify a binding-expression source as "static enough to
    /// inline as a direct DOM assignment". Conservative — any `()`
    /// call, optional-chain `?.`, or nullish-coalescing `??` defers
    /// back to the `bind*` effect path so reactivity / null semantics
    /// stay correct. Only meaningful inside a `{#for}` row body, where
    /// each effect costs a closure + cleanup slot × every row × every
    /// render.
    pub(super) fn is_static_row_expr(&self, source: &str) -> bool {
        // TEMP-DISABLE for A/B bench measurement; revert before commit.
        if std::env::var_os("SUI_D5_DISABLE").is_some() {
            return false;
        }
        self.for_in_scope
            && !source.contains('(')
            && !source.contains("?.")
            && !source.contains("??")
    }

    pub(super) fn fresh(&mut self, prefix: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("__{prefix}{id}")
    }

    /// Allocate the next `b<N>` key. Mirrors `bindings::Builder::push`
    /// — call sites must match the registry pre-pass exactly.
    pub(super) fn next_key(&mut self) -> String {
        self.keys.alloc()
    }

    pub fn emit_top_level(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        self.emit_children(w, instrs, parent_var, cleanups_var);
    }

    pub(super) fn emit_children(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        for instr in instrs {
            self.emit_one(w, instr, parent_var, cleanups_var);
        }
    }

    pub(super) fn emit_one(
        &mut self,
        w: &mut Writer,
        instr: &Instr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        match instr {
            Instr::StaticText { text, .. } => {
                let v = self.fresh("t");
                w.line(&format!(
                    "const {v} = document.createTextNode({});",
                    js_string(text)
                ));
                w.line(&format!("{parent_var}.appendChild({v});"));
            }
            // HTML comments are developer documentation; drop from compiled output.
            Instr::Comment { .. } => {}
            Instr::DynamicText { expr } => {
                // Registry parity: text bindings consume one b<N>.
                let key = self.next_key();
                let v = self.fresh("t");
                let rw = rewrite_for_scope(&expr.source, &[]);
                w.line(&format!("const {v} = document.createTextNode(\"\");"));
                w.line(&format!("{parent_var}.appendChild({v});"));
                if self.is_static_row_expr(&expr.source) {
                    // D5: row-scope static read — skip the bindText
                    // effect and write textContent directly. Saves a
                    // closure + cleanup slot per row per binding. The
                    // assignment is also re-run by the for's
                    // `update()` callback so keyed-row reuse refreshes
                    // the DOM when the row instance changes.
                    let stmt = format!("{v}.textContent = ({rw});");
                    w.line(&stmt);
                    self.inline_update_stmts.push(stmt);
                } else {
                    let fv = self.for_version_comma();
                    w.line(&format!(
                        "{cleanups_var}.push(bindText({v}, () => ({fv}{rw})));"
                    ));
                }
                // Phase 3b.3: at depth 0 in views, also emit a direct
                // hydrate call so SSR'd `<!--sui:text:bN-->…<!--/sui-->`
                // markers wire up without going through the runtime
                // walker / `bindings` manifest. The matching `text`
                // manifest entry is suppressed in `js.rs::emit_bindings`.
                if self.is_view && self.boundary_depth == 0 {
                    self.hydrate_uses_text = true;
                    self.hydrate_calls.line(&format!(
                        "if (__t[{k}]) __cleanups.push(hydrateText(__t[{k}], () => ({rw})));",
                        k = js_string(&key),
                    ));
                }
            }
            Instr::Element(el) => self.emit_element(w, el, parent_var, cleanups_var),
            Instr::If(if_instr) => self.emit_if(w, if_instr, parent_var, cleanups_var),
            Instr::For(for_instr) => self.emit_for(w, for_instr, parent_var, cleanups_var),
            Instr::Match(match_instr) => {
                self.emit_match(w, match_instr, parent_var, cleanups_var)
            }
            // While-loading/while-error: the render callback is emitted as a
            // named property on the setup return object (via `render_callbacks`
            // buffer). `__mount` emits an inline call for the legacy path.
            Instr::WhileLoading(wl) => {
                self.emit_while_loading_block(w, wl, parent_var, cleanups_var)
            }
            Instr::WhileError(we) => {
                self.emit_while_error_block(w, we, parent_var, cleanups_var)
            }
        }
    }

    /// Emit a `{ nodes, cleanup }` branch result. Used by `bindIf`,
    /// `bindFor`, and the `<while-*>` helpers.
    pub(super) fn emit_branch(&mut self, w: &mut Writer, body: &[Instr]) {
        w.line("() => {");
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&format!("const {cl} = [];"));
        self.boundary_depth += 1;
        self.emit_children(w, body, &frag, &cl);
        self.boundary_depth -= 1;
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("},");
    }
}
