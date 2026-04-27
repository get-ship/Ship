//! Phase 8 — ES module emission.
//!
//! Walks the parsed `SailFile` and produces a single `.js` module that
//! defines the component via `@sail/runtime`. The MVP supports the
//! straight-line authoring path:
//!
//! - `prop`, `state`, `derived` declarations
//! - `function` declarations (verbatim)
//! - `mount` / `unmount` lifecycle blocks
//! - Template: elements, text, `{expr}` text interpolations,
//!   interpolated attribute values, `on:event[|mod...]`, `bind:value`,
//!   `bind:checked`
//!
//! Skipped for MVP (errors gracefully): control-flow blocks (`#if`,
//! `#for`, `#await`), `data` declarations, `effect` blocks, `event`
//! declarations, slots, components-as-children with prop bindings,
//! raw HTML.
//!
//! Anchor strategy:
//! - Elements that need wiring get a `data-sa="N"` attribute
//! - Text interpolations get a `<span data-st="N"></span>` placeholder
//!
//! See `docs/sail/02-compiler-architecture.md § Phase 8` for the full
//! emit shape.

use crate::parse::ast::{
    Attr, AttrValue, EventModifier, Expr, Node, Pattern, ScriptBlock, SailFile, Template,
};

/// Emit the JS module source for a parsed Sail file. `tag` is the
/// custom-element tag name (typically the file basename, kebab-cased).
pub fn emit(file: &SailFile, tag: &str) -> String {
    let (js, _html) = emit_with_html(file, tag);
    js
}

/// Emit both the JS module and the static HTML (for SSR reuse).
pub fn emit_with_html(file: &SailFile, tag: &str) -> (String, String) {
    let mut e = Emitter::new(tag);
    if let Some(style) = &file.style {
        e.scoped_css = crate::lower::style::scope_style(style, tag);
    }
    if let Some(s) = &file.script {
        e.collect_reactive_names(s);
    }
    if let Some(t) = &file.template {
        e.emit_template(t);
    }
    let html = e.html.clone();
    let js = e.render(file.script.as_ref());
    (js, html)
}

struct Emitter<'a> {
    tag: &'a str,
    /// Reactive identifier names: state ∪ derived ∪ data ∪ prop names,
    /// plus any branch-scoped names (e.g., `it`, `i` inside a for body).
    reactive: Vec<String>,
    /// HTML being assembled, with anchor markers.
    html: String,
    /// Wiring instructions to emit inside the factory closure.
    wires: Vec<String>,
    /// Counter for `data-sa` anchors on elements (per-emit-context).
    next_anchor: u32,
    /// Counter for `data-st` anchors on text interpolations (per-context).
    next_text_anchor: u32,
    /// Imported names from `@sail/runtime`.
    imports: std::collections::BTreeSet<&'static str>,
    /// JS expression that resolves to the DOM subtree to query against
    /// when wiring bindings — `host` at the top level, `_root` inside
    /// a captured branch.
    query_root: String,
    /// JS expression naming the array we push disposers into —
    /// `cleanups` at the top level, `_c` inside a captured branch.
    cleanup_array: String,
    /// Pre-scoped CSS to inject at module load (empty if no style).
    scoped_css: String,
    /// Slot declarations encountered while walking the template.
    /// Each entry is the JS source for a `SlotDecl` literal.
    slots: Vec<String>,
}

impl<'a> Emitter<'a> {
    fn new(tag: &'a str) -> Self {
        let mut imports: std::collections::BTreeSet<&'static str> = Default::default();
        imports.insert("defineComponent");
        Self {
            tag,
            reactive: Vec::new(),
            html: String::new(),
            wires: Vec::new(),
            next_anchor: 0,
            next_text_anchor: 0,
            imports,
            query_root: "host".into(),
            cleanup_array: "cleanups".into(),
            scoped_css: String::new(),
            slots: Vec::new(),
        }
    }

    fn collect_reactive_names(&mut self, s: &ScriptBlock) {
        for p in &s.props { self.reactive.push(p.name.clone()); }
        for st in &s.state { self.reactive.push(st.name.clone()); }
        for d in &s.derived { self.reactive.push(d.name.clone()); }
        for d in &s.data { self.reactive.push(d.name.clone()); }
    }

    fn emit_template(&mut self, t: &Template) {
        for n in &t.root { self.emit_node(n); }
    }

    fn emit_node(&mut self, n: &Node) {
        match n {
            Node::Text(s) => self.html.push_str(s),
            Node::Interpolation(expr) => {
                let id = self.next_text_anchor;
                self.next_text_anchor += 1;
                self.html.push_str(&format!("<span data-st=\"{id}\"></span>"));
                self.imports.insert("bindText");
                let deps = self.deps_array(&expr.raw);
                let root = self.query_root.clone();
                let cl = self.cleanup_array.clone();
                let wrapped = self.rx(&expr.raw);
                self.wires.push(format!(
                    "  {{\n    const _n = {root}.querySelector('[data-st=\"{id}\"]');\n    if (_n) {cl}.push(bindText(_n, () => String({wrapped}), {deps}));\n  }}",
                ));
            }
            Node::Element { tag, attrs, children } => {
                if tag == "slot" {
                    self.emit_slot_decl(attrs, children);
                } else {
                    self.emit_element(tag, attrs, children);
                }
            }
            Node::Component { tag, props, events, children, .. } => {
                // Prop values are JS expressions — emit as interpolated
                // attributes so the runtime keeps the attribute (and
                // therefore the child component's prop signal) in sync
                // with the dependency.
                let attr_list: Vec<Attr> = props
                    .iter()
                    .map(|p| Attr {
                        name: kebab_attr(&p.name),
                        value: AttrValue::Interpolated(p.value.clone()),
                    })
                    .chain(events.iter().map(|e| Attr {
                        name: format!("on:{}", e.event),
                        value: AttrValue::OnEvent {
                            event: e.event.clone(),
                            modifiers: e.modifiers.clone(),
                            handler: e.handler.clone(),
                        },
                    }))
                    .collect();
                self.emit_element(tag, &attr_list, children);
            }
            // Skipped for MVP — emitter writes a comment placeholder
            // so the author knows the block is recognised but not yet
            // emitted.
            Node::If { cond, then_, elifs, else_ } => {
                self.emit_if(cond, then_, elifs, else_.as_deref());
            }
            Node::For { item, iter, key, body, empty } => {
                self.emit_for(item, iter, key.as_ref(), body, empty.as_deref());
            }
            Node::Await { promise, loading, then_, catch } => {
                self.emit_await(promise, loading, then_.as_ref(), catch.as_ref());
            }
            Node::Slot { .. } | Node::SlotFill { .. } => {
                self.html.push_str("<!-- sail: <slot> not yet emitted -->");
            }
            Node::Window { events } => self.emit_global_events("window", events),
            Node::Document { events } => self.emit_global_events("document", events),
            Node::RawHtml(expr) => {
                let id = self.next_anchor;
                self.next_anchor += 1;
                self.html.push_str(&format!("<span data-sh=\"{id}\"></span>"));
                self.imports.insert("bindRawHtml");
                let deps = self.deps_array(&expr.raw);
                let root = self.query_root.clone();
                let cl = self.cleanup_array.clone();
                let wrapped = self.rx(&expr.raw);
                self.wires.push(format!(
                    "  {{\n    const _n = {root}.querySelector('[data-sh=\"{id}\"]');\n    if (_n) {cl}.push(bindRawHtml(_n, () => ({wrapped}), {deps}));\n  }}",
                ));
            }
            // Compile-time layout primitives. The resolver in
            // `lower::layout` should have replaced these by the time
            // emit runs. If we see them here, the file failed to
            // resolve (no <extends>) — emit the block's default body
            // for blocks, the fill body for orphaned fills, and a
            // comment placeholder for unresolved includes.
            Node::Block { default, .. } => {
                for c in default {
                    self.emit_node(c);
                }
            }
            Node::Fill { body, .. } => {
                for c in body {
                    self.emit_node(c);
                }
            }
            Node::Include { src } => {
                self.html.push_str(&format!(
                    "<!-- sail: unresolved <include src=\"{src}\"> -->"
                ));
            }
        }
    }

    fn emit_element(&mut self, tag: &str, attrs: &[Attr], children: &[Node]) {
        // Decide whether this element needs an anchor.
        let needs_anchor = attrs.iter().any(|a| {
            matches!(
                a.value,
                AttrValue::Interpolated(_) | AttrValue::BindTwoWay(_) | AttrValue::OnEvent { .. }
            )
        });
        let anchor_id = if needs_anchor {
            let id = self.next_anchor;
            self.next_anchor += 1;
            Some(id)
        } else {
            None
        };

        self.html.push('<');
        self.html.push_str(tag);
        if let Some(id) = anchor_id {
            self.html.push_str(&format!(" data-sa=\"{id}\""));
        }
        for a in attrs {
            match &a.value {
                AttrValue::Static(s) => {
                    self.html.push(' ');
                    self.html.push_str(&a.name);
                    self.html.push_str("=\"");
                    self.html.push_str(&escape_attr(s));
                    self.html.push('"');
                }
                AttrValue::Interpolated(expr) => {
                    let id = anchor_id.unwrap();
                    let deps = self.deps_array(&expr.raw);
                    self.imports.insert("bindAttr");
                    let root = self.query_root.clone();
                    let cl = self.cleanup_array.clone();
                    let wrapped = self.rx(&expr.raw);
                    self.wires.push(format!(
                        "  {{\n    const _n = {root}.querySelector('[data-sa=\"{id}\"]');\n    if (_n) {cl}.push(bindAttr(_n, {name:?}, () => ({wrapped}), {deps}));\n  }}",
                        name = a.name,
                    ));
                }
                AttrValue::BindTwoWay(expr) => {
                    let id = anchor_id.unwrap();
                    let bind_fn = match a.name.as_str() {
                        "bind:checked" => "bindChecked",
                        _ => "bindValue",
                    };
                    self.imports.insert(bind_fn);
                    let root = self.query_root.clone();
                    let cl = self.cleanup_array.clone();
                    self.wires.push(format!(
                        "  {{\n    const _n = {root}.querySelector('[data-sa=\"{id}\"]');\n    if (_n) {cl}.push({bind_fn}(_n, {sig}));\n  }}",
                        sig = expr.raw,
                    ));
                }
                AttrValue::OnEvent { event, modifiers, handler } => {
                    let id = anchor_id.unwrap();
                    self.imports.insert("on");
                    let opts = render_modifier_opts(modifiers);
                    let root = self.query_root.clone();
                    let cl = self.cleanup_array.clone();
                    let handler_wrapped = self.rx(&handler.raw);
                    self.wires.push(format!(
                        "  {{\n    const _n = {root}.querySelector('[data-sa=\"{id}\"]');\n    if (_n) {cl}.push(on(_n, {event:?}, {handler_wrapped}{opts}));\n  }}",
                    ));
                }
            }
        }
        self.html.push('>');
        for c in children { self.emit_node(c); }
        self.html.push_str("</");
        self.html.push_str(tag);
        self.html.push('>');
    }

    /// Capture a sub-tree's HTML + wires into independent buffers.
    /// `extra_scope` adds names treated as reactive only inside the
    /// captured walk (for `{#for}`'s `it` / `i`).
    fn capture(
        &mut self,
        nodes: &[Node],
        extra_scope: &[String],
    ) -> (String, Vec<String>) {
        let saved_html = std::mem::take(&mut self.html);
        let saved_wires = std::mem::take(&mut self.wires);
        let saved_root = std::mem::replace(&mut self.query_root, "_root".into());
        let saved_cl = std::mem::replace(&mut self.cleanup_array, "_c".into());
        // Anchor counters are shared across contexts so IDs are
        // globally unique within the file. Without this, an outer
        // host.querySelector('[data-st="0"]') can match a branch's
        // anchor that appears earlier in document order once the
        // branch renders.
        for s in extra_scope { self.reactive.push(s.clone()); }

        for n in nodes { self.emit_node(n); }

        let captured_html = std::mem::replace(&mut self.html, saved_html);
        let captured_wires = std::mem::replace(&mut self.wires, saved_wires);
        self.query_root = saved_root;
        self.cleanup_array = saved_cl;
        for _ in 0..extra_scope.len() { self.reactive.pop(); }

        (captured_html, captured_wires)
    }

    /// Emit a JS arrow that builds a branch — used by both `{#if}` and
    /// `{#for}`. Returns source like:
    /// `() => { const _tpl = ...; const _root = ...; const _c = []; ...; return { nodes, cleanup, ...extra } }`
    fn render_branch(
        &self,
        html: &str,
        wires: &[String],
        prelude: &str,
        extra_return: &str,
    ) -> String {
        let mut out = String::from("() => {\n");
        out.push_str(prelude);
        out.push_str("      const _tpl = document.createElement('template')\n");
        out.push_str(&format!(
            "      _tpl.innerHTML = {}\n",
            render_template_literal(html)
        ));
        out.push_str("      const _root = _tpl.content\n");
        out.push_str("      const _c = []\n");
        for w in wires {
            // Re-indent each captured wire to fit inside the closure.
            for line in w.lines() {
                out.push_str("      ");
                out.push_str(line.trim_start());
                out.push('\n');
            }
        }
        // Snapshot _nodes AFTER wires run — wires may have mutated
        // _root by replacing slot/if/for placeholders with comment
        // anchors, and the returned node list must reflect those.
        out.push_str("      const _nodes = Array.from(_root.childNodes)\n");
        out.push_str(&format!(
            "      return {{ nodes: _nodes, cleanup: () => _c.forEach(c => {{ try {{ c() }} catch {{}} }}){extra_return} }}\n"
        ));
        out.push_str("    }");
        out
    }

    fn emit_if(
        &mut self,
        cond: &Expr,
        then_: &[Node],
        elifs: &[(Expr, Vec<Node>)],
        else_: Option<&[Node]>,
    ) {
        let id = self.next_anchor;
        self.next_anchor += 1;
        self.html.push_str(&format!("<template data-si=\"{id}\"></template>"));
        self.imports.insert("bindIf");

        // Fold elifs into nested else-bindIfs so the runtime stays simple.
        // For shape `if A then a else if B then b else c`, the JS is:
        //   bindIf({ cond: A, then_: a, else_: () => bindIf({...}) })
        // To keep this manageable we render the whole chain directly.

        let (then_html, then_wires) = self.capture(then_, &[]);
        let then_arrow = self.render_branch(&then_html, &then_wires, "", "");

        // Build else_ expression — could be a chain.
        let else_expr = self.build_else_chain(elifs, else_);

        let root = self.query_root.clone();
        let cl = self.cleanup_array.clone();
        let cond_deps = self.deps_array(&cond.raw);
        let mut wire = String::from("  {\n");
        wire.push_str(&format!(
            "    const _slot = {root}.querySelector('[data-si=\"{id}\"]')\n"
        ));
        wire.push_str("    const _anchor = document.createComment('si')\n");
        wire.push_str("    if (_slot) _slot.replaceWith(_anchor)\n");
        let cond_wrapped = self.rx(&cond.raw);
        wire.push_str(&format!(
            "    {cl}.push(bindIf({{\n      anchor: _anchor,\n      cond: () => ({cond_wrapped}),\n      then_: {then_arrow}",
        ));
        if let Some(else_arrow) = else_expr {
            wire.push_str(&format!(",\n      else_: {else_arrow}"));
        }
        let _ = cond_deps; // bindIf reactivity is via effect inside it; deps not needed here
        wire.push_str("\n    }))\n  }");
        self.wires.push(wire);
    }

    fn build_else_chain(
        &mut self,
        elifs: &[(Expr, Vec<Node>)],
        else_: Option<&[Node]>,
    ) -> Option<String> {
        if elifs.is_empty() {
            return match else_ {
                Some(nodes) => {
                    let (h, w) = self.capture(nodes, &[]);
                    Some(self.render_branch(&h, &w, "", ""))
                }
                None => None,
            };
        }
        // Take the head elif → emit as `() => { ... bindIf(...) ... }`
        // returning a single fragment containing the inner bindIf's
        // current branch nodes. Because bindIf is anchor-based, we
        // need an inner anchor inside this branch.
        let (head_cond, head_body) = (&elifs[0].0, &elifs[0].1);
        let (inner_then_html, inner_then_wires) = self.capture(head_body, &[]);
        let inner_then_arrow =
            self.render_branch(&inner_then_html, &inner_then_wires, "", "");
        let inner_else = self.build_else_chain(&elifs[1..], else_);
        let head_cond_wrapped = self.rx(&head_cond.raw);

        // Inner bindIf renders into a fragment that contains both the
        // anchor and any branch content. Returning ALL of the
        // fragment's children (not just the anchor) lets the outer
        // caller move them as a unit when placing — bindIf's effect
        // continues to work after placement because the anchor
        // travels with its inserted siblings.
        let mut arrow = String::from("() => {\n");
        arrow.push_str("      const _anchor = document.createComment('si-elif')\n");
        arrow.push_str("      const _frag = document.createDocumentFragment()\n");
        arrow.push_str("      _frag.appendChild(_anchor)\n");
        arrow.push_str(&format!(
            "      const _d = bindIf({{ anchor: _anchor, cond: () => ({head_cond_wrapped}), then_: {then_} ",
            then_ = inner_then_arrow,
        ));
        if let Some(e) = inner_else {
            arrow.push_str(&format!(", else_: {e}"));
        }
        arrow.push_str(" })\n");
        arrow.push_str("      return { nodes: Array.from(_frag.childNodes), cleanup: _d }\n");
        arrow.push_str("    }");
        Some(arrow)
    }

    fn emit_slot_decl(&mut self, attrs: &[Attr], fallback: &[Node]) {
        let name = attrs
            .iter()
            .find(|a| a.name == "name")
            .and_then(|a| match &a.value {
                AttrValue::Static(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string());

        let id = self.next_anchor;
        self.next_anchor += 1;
        self.html.push_str(&format!("<template data-ss=\"{id}\"></template>"));
        self.imports.insert("resolveSlots");
        self.imports.insert("snapshotChildren");

        // Capture fallback as a render arrow returning Node[].
        let fallback_arrow = if fallback.is_empty() {
            "undefined".to_string()
        } else {
            let (h, w) = self.capture(fallback, &[]);
            // Build a `() => Node[]` (no cleanup tracking — fallback
            // is a one-shot static render).
            let mut out = String::from("() => {\n");
            out.push_str("      const _tpl = document.createElement('template')\n");
            out.push_str(&format!(
                "      _tpl.innerHTML = {}\n",
                render_template_literal(&h)
            ));
            out.push_str("      const _root = _tpl.content\n");
            out.push_str("      const _c = []\n");
            for line in w.iter().flat_map(|s| s.lines()) {
                out.push_str("      ");
                out.push_str(line.trim_start());
                out.push('\n');
            }
            // Snapshot AFTER wires — bindIf / bindFor wires inside the
            // fallback replace their template placeholder with a Comment
            // anchor at wire-time. The returned node list MUST contain
            // that live anchor or `resolveSlots` splices in stale element
            // refs that no longer have a parent — the bindIf's `then_`
            // branch then renders into an orphaned fragment and never
            // shows up in the live tree. Mirrors `render_branch`'s order.
            out.push_str("      const _nodes = Array.from(_root.childNodes)\n");
            out.push_str("      return _nodes\n");
            out.push_str("    }");
            out
        };

        self.slots.push(format!(
            "{{ name: {name:?}, anchor: _slotAnchors[{id}], fallback: {fb} }}",
            fb = fallback_arrow,
        ));
    }

    fn emit_global_events(&mut self, target: &str, events: &[crate::parse::ast::EventBinding]) {
        if events.is_empty() { return; }
        self.imports.insert("on");
        let cl = self.cleanup_array.clone();
        let mut wire = String::from("  {\n");
        for e in events {
            let opts = render_modifier_opts(&e.modifiers);
            let h = self.rx(&e.handler.raw);
            wire.push_str(&format!(
                "    {cl}.push(on({target}, {ev:?}, {h}{opts}));\n",
                ev = e.event,
            ));
        }
        wire.push_str("  }");
        self.wires.push(wire);
    }

    fn emit_await(
        &mut self,
        promise: &Expr,
        loading: &[Node],
        then_: Option<&(Pattern, Vec<Node>)>,
        catch: Option<&(Pattern, Vec<Node>)>,
    ) {
        let id = self.next_anchor;
        self.next_anchor += 1;
        self.html.push_str(&format!("<template data-sw=\"{id}\"></template>"));
        self.imports.insert("bindAwait");

        let (loading_html, loading_wires) = self.capture(loading, &[]);
        let loading_arrow = self.render_branch(&loading_html, &loading_wires, "", "");

        let then_arrow = then_.map(|(pat, body)| {
            let name = match pat {
                Pattern::Identifier(n) => n.clone(),
                Pattern::Tuple(parts) => parts.first().cloned().unwrap_or_else(|| "_v".into()),
            };
            let scope = vec![name.clone()];
            let (h, w) = self.capture(body, &scope);
            // Need to expose `name` as a signal so inner bindings react.
            self.imports.insert("signal");
            let prelude = format!("      const {name} = signal(_v)\n");
            let arrow = self.render_branch(&h, &w, &prelude, "");
            arrow.replacen("() => {", "(_v) => {", 1)
        });

        let catch_arrow = catch.map(|(pat, body)| {
            let name = match pat {
                Pattern::Identifier(n) => n.clone(),
                Pattern::Tuple(parts) => parts.first().cloned().unwrap_or_else(|| "_e".into()),
            };
            let scope = vec![name.clone()];
            let (h, w) = self.capture(body, &scope);
            self.imports.insert("signal");
            let prelude = format!("      const {name} = signal(_e)\n");
            let arrow = self.render_branch(&h, &w, &prelude, "");
            arrow.replacen("() => {", "(_e) => {", 1)
        });

        let root = self.query_root.clone();
        let cl = self.cleanup_array.clone();
        let mut wire = String::from("  {\n");
        wire.push_str(&format!(
            "    const _slot = {root}.querySelector('[data-sw=\"{id}\"]')\n"
        ));
        wire.push_str("    const _anchor = document.createComment('sw')\n");
        wire.push_str("    if (_slot) _slot.replaceWith(_anchor)\n");
        let p_wrapped = self.rx(&promise.raw);
        wire.push_str(&format!(
            "    {cl}.push(bindAwait({{\n      anchor: _anchor,\n      promise: () => ({p_wrapped}),\n      loading: {loading}",
            loading = loading_arrow,
        ));
        if let Some(t) = then_arrow { wire.push_str(&format!(",\n      then_: {t}")); }
        if let Some(c) = catch_arrow { wire.push_str(&format!(",\n      catch_: {c}")); }
        wire.push_str("\n    }))\n  }");
        self.wires.push(wire);
    }

    fn emit_for(
        &mut self,
        item: &Pattern,
        iter: &Expr,
        key: Option<&Expr>,
        body: &[Node],
        empty: Option<&[Node]>,
    ) {
        let id = self.next_anchor;
        self.next_anchor += 1;
        self.html.push_str(&format!("<template data-sf=\"{id}\"></template>"));
        self.imports.insert("bindFor");
        self.imports.insert("signal");

        let (item_name, idx_name) = match item {
            Pattern::Identifier(n) => (n.clone(), "_i".to_string()),
            Pattern::Tuple(parts) => (
                parts.first().cloned().unwrap_or_else(|| "it".into()),
                parts.get(1).cloned().unwrap_or_else(|| "_i".into()),
            ),
        };

        let scope = vec![item_name.clone(), idx_name.clone()];
        let (body_html, body_wires) = self.capture(body, &scope);

        let prelude = format!(
            "      const {item} = signal(_initialItem)\n      const {idx} = signal(_initialIdx)\n",
            item = item_name,
            idx = idx_name,
        );
        let extra = format!(
            ", update: (nextItem, nextIdx) => {{ {item}.set(nextItem); {idx}.set(nextIdx) }}",
            item = item_name,
            idx = idx_name,
        );

        // render_branch emits `() => { ... }`, but bindFor.render takes
        // `(item, index) => RenderedItem`. Rewrite the leading arrow.
        let body_arrow = self.render_branch(&body_html, &body_wires, &prelude, &extra);
        let body_arrow = body_arrow.replacen("() => {", "(_initialItem, _initialIdx) => {", 1);

        // Key function parameters mirror the user's loop pattern names
        // so a `key={i}` clause referencing the index works. The body
        // is unwrapped (item is the raw value here, not a signal) so
        // member access reads naturally — `key={item.id}`.
        let key_expr = match key {
            Some(k) => format!(
                "({item}, {idx}) => ({k})",
                item = item_name,
                idx = idx_name,
                k = k.raw,
            ),
            None => format!(
                "({item}, {idx}) => {idx}",
                item = item_name,
                idx = idx_name,
            ),
        };

        // Optionally capture the empty branch BEFORE we emit the wire
        // so both the for and the empty placement share the same
        // _slot reference.
        let empty_arrow = empty.map(|empty_nodes| {
            let (h, w) = self.capture(empty_nodes, &[]);
            self.imports.insert("bindIf");
            self.render_branch(&h, &w, "", "")
        });

        let root = self.query_root.clone();
        let cl = self.cleanup_array.clone();
        let iter_wrapped = self.rx(&iter.raw);
        let mut wire = String::from("  {\n");
        wire.push_str(&format!(
            "    const _slot = {root}.querySelector('[data-sf=\"{id}\"]')\n"
        ));
        wire.push_str("    const _anchor = document.createComment('sf')\n");
        if let Some(empty) = &empty_arrow {
            wire.push_str("    const _eAnchor = document.createComment('sf-empty')\n");
            wire.push_str("    if (_slot) {\n");
            wire.push_str("      _slot.parentNode.insertBefore(_anchor, _slot)\n");
            wire.push_str("      _slot.parentNode.insertBefore(_eAnchor, _slot)\n");
            wire.push_str("      _slot.remove()\n");
            wire.push_str("    }\n");
            wire.push_str(&format!(
                "    {cl}.push(bindIf({{ anchor: _eAnchor, cond: () => ({iter_wrapped}).length === 0, then_: {empty} }}))\n",
                empty = empty,
            ));
        } else {
            wire.push_str("    if (_slot) _slot.replaceWith(_anchor)\n");
        }
        wire.push_str(&format!(
            "    {cl}.push(bindFor({{\n      anchor: _anchor,\n      items: () => ({iter_wrapped}),\n      key: {key},\n      render: {body}\n    }}))\n",
            key = key_expr,
            body = body_arrow,
        ));
        wire.push_str("  }");
        self.wires.push(wire);
    }

    /// Auto-wrap reactive identifiers in an expression as calls.
    /// Convenience that picks up the current scope's reactive set.
    fn rx(&self, expr: &str) -> String {
        wrap_reactive_calls(expr, &self.reactive)
    }

    fn deps_array(&mut self, expr: &str) -> String {
        let used: Vec<&String> = self
            .reactive
            .iter()
            .filter(|name| expr_references(expr, name))
            .collect();
        format!("[{}]", used.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
    }

    fn render(&mut self, script: Option<&ScriptBlock>) -> String {
        let mut imports: Vec<&'static str> = self.imports.iter().copied().collect();
        // For decoded prop signal seeding.
        if let Some(s) = script {
            if !s.props.is_empty() { imports.push("signal"); }
            if !s.state.is_empty() { imports.push("signal"); }
            if !s.derived.is_empty() { imports.push("derived"); }
            if s.data.iter().any(|d| !d.live) { imports.push("signal"); }
            if s.data.iter().any(|d| d.live) { imports.push("data"); }
            if s.effects.iter().any(|e| !e.once) { imports.push("effect"); }
        }
        imports.sort();
        imports.dedup();

        let mut out = String::new();
        out.push_str("// Auto-generated by Sail compiler. Do not edit.\n");
        out.push_str(&format!(
            "import {{ {} }} from '@sail/runtime'\n\n",
            imports.join(", ")
        ));

        // Schema (observed attrs + decoders).
        out.push_str("const SCHEMA = ");
        out.push_str(&render_schema(script));
        out.push_str("\n\n");

        // Static HTML.
        out.push_str("const HTML = ");
        out.push_str(&render_template_literal(&self.html));
        out.push_str("\n\n");

        // Scoped style (one-shot inject at module load).
        let scoped = &self.scoped_css;
        if !scoped.is_empty() {
            out.push_str("const STYLE = ");
            out.push_str(&render_template_literal(scoped));
            out.push_str("\n");
            out.push_str(&format!(
                "if (typeof document !== 'undefined' && !document.head.querySelector('style[data-sail={tag:?}]')) {{\n  const _s = document.createElement('style')\n  _s.setAttribute('data-sail', {tag:?})\n  _s.textContent = STYLE\n  document.head.appendChild(_s)\n}}\n\n",
                tag = self.tag,
            ));
        }

        // Factory. Form-associated components get `internals` as a
        // third parameter (an ElementInternals instance attached at
        // construction by the runtime); other components don't pay
        // for the unused signature.
        let form_associated = script.map(|s| s.form_associated).unwrap_or(false);
        let factory_params = if form_associated {
            "(host, propSignals, internals)"
        } else {
            "(host, propSignals)"
        };
        out.push_str(&format!(
            "defineComponent({tag:?}, SCHEMA, {factory_params} => {{\n",
            tag = self.tag
        ));
        // `el` is the spec-reserved alias for the host element. Bind it
        // up front so script-block code (mount/unmount blocks, user
        // functions, effect bodies) can reference it.
        out.push_str("  const el = host\n");
        out.push_str("  const cleanups = []\n");

        if let Some(s) = script {
            // Props: seed each signal from the current attribute (if
            // present) so values set BEFORE connection survive. The
            // browser fires attributeChangedCallback during upgrade
            // (before connectedCallback runs the factory), so without
            // this seed pre-set attrs are lost on the way to the
            // signals — they don't exist yet to receive .set().
            for p in &s.props {
                let kebab = kebab_of(&p.name);
                let initial = p.default.as_deref().unwrap_or("undefined");
                out.push_str(&format!(
                    "  const {name} = propSignals.{name} = signal(\n    host.hasAttribute({kebab:?}) ? SCHEMA.props[{kebab:?}].decode(host.getAttribute({kebab:?})) : ({initial})\n  )\n",
                    name = p.name,
                ));
            }
            // State & derived.
            for st in &s.state {
                let initial = self.rx(&st.initial);
                out.push_str(&format!(
                    "  const {name} = signal({initial})\n",
                    name = st.name,
                ));
            }
            for d in &s.derived {
                let expr = self.rx(&d.expr);
                out.push_str(&format!(
                    "  const {name} = derived(() => ({expr}))\n",
                    name = d.name,
                ));
            }
            // `data X = expr`        → signal seeded by expr (static; analyzer
            //                          phase can swap to a server-resolved value)
            // `data live X = expr`   → data({ initial: null, subscribe }) —
            //                          the expression is a server-side
            //                          schema source (e.g. `ship.agents.summaries()`),
            //                          NOT a JS expression. Inlining it
            //                          as JS would throw `ReferenceError`
            //                          at instantiation. The runtime opens
            //                          the SSE stream and the first patch
            //                          carries the real initial value.
            for d in &s.data {
                if d.live {
                    // `data` import is emitted in the prelude — see render().
                    out.push_str(&format!(
                        "  const _d_{name} = data({{ initial: null, subscribe: {url:?} }})\n  const {name} = _d_{name}.signal\n  cleanups.push(_d_{name}.dispose)\n",
                        name = d.name,
                        url = format!("/_sail/data?expr={}", d.expr),
                    ));
                } else {
                    let expr = self.rx(&d.expr);
                    out.push_str(&format!(
                        "  const {name} = signal(({expr}))\n",
                        name = d.name,
                    ));
                }
            }
            // User functions verbatim.
            for f in &s.functions {
                out.push_str("  ");
                out.push_str(&f.raw.replace('\n', "\n  "));
                out.push('\n');
            }
            // `effect [once] [on a, b] { body }` declarations.
            for ef in &s.effects {
                let body = ef.body.replace('\n', "\n    ");
                if ef.once {
                    out.push_str(&format!(
                        "  // effect once {{}}\n  {{\n    {body}\n  }}\n"
                    ));
                } else {
                    let deps = if ef.deps.is_empty() {
                        "[]".to_string()
                    } else {
                        format!("[{}]", ef.deps.join(", "))
                    };
                    self.imports.insert("effect");
                    out.push_str(&format!(
                        "  cleanups.push(effect(() => {{\n    {body}\n  }}, {deps}))\n"
                    ));
                }
            }
        }

        // Slots: snapshot children BEFORE innerHTML so light-DOM fills survive.
        if !self.slots.is_empty() {
            out.push_str("  const _origChildren = snapshotChildren(host)\n");
        }
        out.push_str("  host.innerHTML = HTML\n");
        if !self.slots.is_empty() {
            out.push_str("  const _slotAnchors = []\n");
            // Replace each <template data-ss="N"> with a Comment anchor.
            // The slot id is the value of data-ss, indexed into _slotAnchors.
            out.push_str("  for (const _t of host.querySelectorAll('[data-ss]')) {\n");
            out.push_str("    const _idx = Number(_t.getAttribute('data-ss'))\n");
            out.push_str("    const _a = document.createComment('ss')\n");
            out.push_str("    _t.replaceWith(_a)\n");
            out.push_str("    _slotAnchors[_idx] = _a\n");
            out.push_str("  }\n");
            out.push_str(&format!(
                "  resolveSlots(host, [{}], _origChildren)\n",
                self.slots.join(", ")
            ));
        }

        for w in &self.wires {
            out.push_str(w);
            out.push('\n');
        }

        if let Some(s) = script {
            if let Some(mount) = &s.mount {
                out.push_str("  // mount {} block\n  {");
                out.push_str(&mount.replace('\n', "\n  "));
                out.push_str("}\n");
            }
        }

        out.push_str("  return () => {\n");
        out.push_str("    for (const c of cleanups) try { c() } catch {}\n");
        if let Some(s) = script {
            if let Some(unmount) = &s.unmount {
                out.push_str("    // unmount {} block\n    {");
                out.push_str(&unmount.replace('\n', "\n    "));
                out.push_str("}\n");
            }
        }
        out.push_str("  }\n");
        out.push_str("})\n");
        out
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

fn render_template_literal(s: &str) -> String {
    // Use a backtick template literal; escape backslashes, backticks,
    // and `${`.
    let mut out = String::with_capacity(s.len() + 2);
    out.push('`');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '$' => out.push_str("\\$"),
            _ => out.push(c),
        }
    }
    out.push('`');
    out
}

fn render_schema(script: Option<&ScriptBlock>) -> String {
    let Some(s) = script else { return "{ observedAttrs: [], props: {} }".to_string() };
    let form_assoc_suffix = if s.form_associated {
        ", formAssociated: true"
    } else {
        ""
    };
    if s.props.is_empty() {
        return format!("{{ observedAttrs: [], props: {{}}{form_assoc_suffix} }}");
    }
    let mut out = String::from("{ observedAttrs: [");
    for (i, p) in s.props.iter().enumerate() {
        if i > 0 { out.push_str(", "); }
        out.push_str(&format!("{:?}", kebab_of(&p.name)));
    }
    out.push_str("], props: {");
    for (i, p) in s.props.iter().enumerate() {
        if i > 0 { out.push_str(", "); }
        let decoder = decoder_for(p.ty.as_deref());
        out.push_str(&format!(
            " {kebab:?}: {{ camel: {camel:?}, decode: {decoder} }}",
            kebab = kebab_of(&p.name),
            camel = p.name,
        ));
    }
    out.push_str(" }");
    out.push_str(form_assoc_suffix);
    out.push_str(" }");
    out
}

fn decoder_for(ty: Option<&str>) -> &'static str {
    let Some(ty) = ty.map(str::trim) else { return "(r) => r ?? ''" };
    match ty {
        "number" => "(r) => r === null || r === '' ? NaN : Number(r)",
        "boolean" => "(r) => r !== null && r.toLowerCase() !== 'false'",
        "string" => "(r) => r ?? ''",
        _ => {
            // Arrays (`T[]`, `Array<T>`), object literals (`{ ... }`),
            // and unrecognized types decode as JSON. Falls back to the
            // raw string if parse fails so a typo doesn't crash the
            // component.
            if ty.ends_with("[]")
                || ty.starts_with("Array<")
                || ty.starts_with('{')
                || ty.starts_with("Record<")
                || ty.starts_with("Map<")
                || ty.starts_with("Set<")
            {
                "(r) => { if (r === null || r === '') return null; try { return JSON.parse(r) } catch { return r } }"
            } else {
                // String literal unions (`'a' | 'b'`), function types,
                // and other type expressions: pass through as string.
                "(r) => r ?? ''"
            }
        }
    }
}

/// Convert a camelCase prop name to its kebab-case attribute form.
fn kebab_attr(name: &str) -> String {
    kebab_of(name)
}

fn kebab_of(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 { out.push('-'); }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn render_modifier_opts(mods: &[EventModifier]) -> String {
    if mods.is_empty() { return String::new(); }
    let parts: Vec<String> = mods
        .iter()
        .map(|m| match &m.arg {
            Some(a) => format!("{{ kind: {:?}, arg: {:?} }}", m.kind, a),
            None => format!("{{ kind: {:?} }}", m.kind),
        })
        .collect();
    format!(", {{ modifiers: [{}] }}", parts.join(", "))
}

/// Auto-call reactive signal identifiers in an expression.
///
/// In Sail's template syntax, `{count}` means "read the value of the
/// `count` signal". The runtime exposes signals as getter functions
/// (`count()`), so every bare reactive identifier in an expression
/// must be rewritten into a call. This walks the expression
/// string-aware and member-access-aware, wrapping each occurrence of
/// a name in `reactive` as `<name>()` unless:
///
/// - it's already followed by `(` (already a call)
/// - it's followed by `.set` or `.subscribe` (signal API methods)
/// - it appears after `.` (member-access RHS — not the same identifier)
///
/// All other shapes — `count + 1`, `items.length`, `user.name`,
/// `String(count)` — get the wrap.
fn wrap_reactive_calls(expr: &str, reactive: &[String]) -> String {
    let bytes = expr.as_bytes();
    let mut out = String::with_capacity(expr.len() + 16);
    let mut i = 0usize;
    let mut in_str: Option<u8> = None;
    let mut after_dot = false;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            // Template literal `…${ expr }…` — when we see `${`, switch
            // back to expression mode (tracking brace balance) so
            // reactive idents inside the interpolation get wrapped.
            // Plain `'`/`"` strings don't have interpolation so this
            // only matters for backticks.
            if q == b'`' && b == b'$' && bytes.get(i + 1) == Some(&b'{') {
                out.push('$');
                out.push('{');
                i += 2;
                // Recursively wrap the interpolation body. Find the
                // matching `}` honoring nested braces, strings, and
                // nested template literals.
                let inner_start = i;
                let mut depth = 1i32;
                while i < bytes.len() && depth > 0 {
                    let c = bytes[i];
                    if c == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    if c == b'`' {
                        // Nested template literal — skip to its close.
                        i += 1;
                        while i < bytes.len() {
                            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                i += 2;
                                continue;
                            }
                            if bytes[i] == b'`' {
                                i += 1;
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    if c == b'"' || c == b'\'' {
                        let q2 = c;
                        i += 1;
                        while i < bytes.len() && bytes[i] != q2 {
                            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                i += 2;
                                continue;
                            }
                            i += 1;
                        }
                        i += 1;
                        continue;
                    }
                    if c == b'{' { depth += 1; }
                    if c == b'}' {
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    i += 1;
                }
                let inner = &expr[inner_start..i];
                out.push_str(&wrap_reactive_calls(inner, reactive));
                if i < bytes.len() {
                    out.push('}');
                    i += 1;
                }
                continue;
            }
            out.push(b as char);
            if b == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if b == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            in_str = Some(b);
            out.push(b as char);
            i += 1;
            continue;
        }
        // Comments — pass through verbatim without wrapping anything
        // inside. Line comments end at \n; block comments at */.
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                while i < bytes.len() && bytes[i] != b'\n' {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                out.push('/');
                out.push('*');
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    out.push('*');
                    out.push('/');
                    i += 2;
                }
                continue;
            }
        }
        if b == b'.' {
            after_dot = true;
            out.push('.');
            i += 1;
            continue;
        }
        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }
            let ident = &expr[start..i];
            out.push_str(ident);
            if !after_dot && reactive.iter().any(|r| r == ident) {
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let already_called = bytes.get(j) == Some(&b'(');
                let is_signal_method = bytes.get(j) == Some(&b'.')
                    && (expr[j..].starts_with(".set") || expr[j..].starts_with(".subscribe"));
                if !already_called && !is_signal_method {
                    out.push_str("()");
                }
            }
            after_dot = false;
            continue;
        }
        if !b.is_ascii_whitespace() {
            after_dot = false;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Best-effort word-boundary search for `name` in `expr` to decide
/// whether the expression depends on a reactive declaration. Skips
/// occurrences inside string and template literals.
fn expr_references(expr: &str, name: &str) -> bool {
    let bytes = expr.as_bytes();
    let nbytes = name.as_bytes();
    let mut i = 0usize;
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if b == q { in_str = None; }
            i += 1; continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            in_str = Some(b); i += 1; continue;
        }
        if i + nbytes.len() <= bytes.len() && &bytes[i..i + nbytes.len()] == nbytes {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after_ok = i + nbytes.len() >= bytes.len() || !is_ident_byte(bytes[i + nbytes.len()]);
            if before_ok && after_ok { return true; }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn compile(src: &str) -> String {
        let file = parse::parse(src).unwrap();
        emit(&file, "app-counter")
    }

    #[test]
    fn empty_template_only() {
        let js = compile("<template><div>hi</div></template>");
        assert!(js.contains("defineComponent(\"app-counter\""));
        assert!(js.contains("<div>hi</div>"));
        assert!(js.contains("host.innerHTML = HTML"));
    }

    #[test]
    fn text_interpolation_creates_anchor_and_binding() {
        let js = compile(
            "<script>state count = 0</script>\n<template><span>{count}</span></template>",
        );
        assert!(js.contains("data-st=\"0\""));
        assert!(js.contains("bindText"));
        assert!(js.contains("const count = signal(0)"));
        assert!(js.contains("[count]"));
    }

    #[test]
    fn on_click_wires_handler() {
        let js = compile(
            "<script>state count = 0\nfunction inc() { count.set(count() + 1) }</script>\n<template><button on:click={inc}>+</button></template>",
        );
        assert!(js.contains("data-sa=\"0\""));
        assert!(js.contains("on(_n, \"click\", inc)"));
        assert!(js.contains("function inc()"));
    }

    #[test]
    fn bind_value_emits_bind_value() {
        let js = compile(
            "<script>state draft = ''</script>\n<template><input bind:value={draft} /></template>",
        );
        assert!(js.contains("bindValue(_n, draft)"));
    }

    #[test]
    fn bind_checked_emits_bind_checked() {
        let js = compile(
            "<script>state ok = false</script>\n<template><input bind:checked={ok} /></template>",
        );
        assert!(js.contains("bindChecked(_n, ok)"));
    }

    #[test]
    fn derived_emits_derived_call() {
        let js = compile(
            "<script>state c = 0\nderived d = c * 2</script>\n<template><span>{d}</span></template>",
        );
        assert!(js.contains("const d = derived(() => (c() * 2))"));
        assert!(js.contains("[d]") || js.contains("[c, d]"));
    }

    #[test]
    fn prop_decl_creates_signal_and_schema() {
        let js = compile(
            "<script>prop label: string = 'hi'</script>\n<template><span>{label}</span></template>",
        );
        assert!(js.contains("propSignals.label = signal("));
        assert!(js.contains("'hi'")); // default fallback present
        assert!(js.contains("observedAttrs: [\"label\"]"));
    }

    #[test]
    fn modifier_options_serialized() {
        let js = compile(
            "<script>function go(){}</script>\n<template><a on:click|preventDefault={go}>x</a></template>",
        );
        assert!(js.contains("{ kind: \"preventDefault\" }"));
    }

    #[test]
    fn modifier_with_arg() {
        let js = compile(
            "<script>function go(){}</script>\n<template><input on:input|debounce:300={go} /></template>",
        );
        assert!(js.contains("{ kind: \"debounce\", arg: \"300\" }"));
    }

    #[test]
    fn static_html_unchanged() {
        let js = compile("<template><div class=\"static\">hello</div></template>");
        assert!(js.contains("<div class=\"static\">hello</div>"));
        assert!(!js.contains("data-sa"));
    }

    #[test]
    fn lifecycle_blocks_emit() {
        let js = compile(
            "<script>state x = 0\nmount { console.log('m') }\nunmount { console.log('u') }</script>\n<template><div>{x}</div></template>",
        );
        assert!(js.contains("// mount {} block"));
        assert!(js.contains("console.log('m')"));
        assert!(js.contains("// unmount {} block"));
        assert!(js.contains("console.log('u')"));
    }

    #[test]
    fn end_to_end_counter() {
        let src = r#"<script>
state count = 0
function inc() { count.set(count() + 1) }
</script>
<template>
<button on:click={inc}>Clicks: {count}</button>
</template>"#;
        let js = compile(src);
        // Imports include everything we used
        assert!(js.contains("defineComponent"));
        assert!(js.contains("signal"));
        assert!(js.contains("bindText"));
        assert!(js.contains("on"));
        // Wiring present
        assert!(js.contains("on(_n, \"click\", inc)"));
        assert!(js.contains("bindText"));
        // Static template chunks survive
        assert!(js.contains("Clicks: "));
    }

    #[test]
    fn if_block_emits_bindIf() {
        let js = compile(
            "<script>state on = true</script>\n<template>{#if on}<p>Y</p>{:else}<p>N</p>{/if}</template>",
        );
        assert!(js.contains("bindIf"));
        assert!(js.contains("data-si=\"0\""));
        assert!(js.contains("cond: () => (on())"));
        assert!(js.contains("then_: () =>"));
        assert!(js.contains("else_: () =>"));
    }

    #[test]
    fn if_without_else_omits_else() {
        let js = compile(
            "<script>state on = true</script>\n<template>{#if on}<p>Y</p>{/if}</template>",
        );
        assert!(js.contains("bindIf"));
        assert!(!js.contains("else_:"));
    }

    #[test]
    fn for_block_emits_bindFor_with_signal_wrapping() {
        let js = compile(
            "<script>state list = []</script>\n<template>{#for it of list key={it.id}}<li>{it.name}</li>{/for}</template>",
        );
        assert!(js.contains("bindFor"));
        assert!(js.contains("data-sf=\"0\""));
        assert!(js.contains("items: () => (list())"));
        assert!(js.contains("key: (it, _i) => (it.id)"));
        assert!(js.contains("const it = signal(_initialItem)"));
        assert!(js.contains("update: (nextItem, nextIdx)"));
        // Inner bindText reads it() and tracks `it` as a dep.
        assert!(js.contains("String(it().name)"));
        assert!(js.contains("[it]"));
    }

    #[test]
    fn for_with_empty_emits_bindIf_for_empty_branch() {
        let js = compile(
            "<script>state xs = []</script>\n<template>{#for x of xs}<li>{x}</li>{:empty}<p>none</p>{/for}</template>",
        );
        assert!(js.contains("bindFor"));
        assert!(js.contains("bindIf"));
        assert!(js.contains("cond: () => (xs()).length === 0"));
        assert!(js.contains("_eAnchor"));
        // Both anchors inserted before _slot is removed.
        assert!(js.contains("insertBefore(_anchor"));
        assert!(js.contains("insertBefore(_eAnchor"));
    }

    #[test]
    fn raw_html_emits_bindRawHtml() {
        let js = compile(
            "<script>state md = '<b>x</b>'</script>\n<template><div>{@html md}</div></template>",
        );
        assert!(js.contains("bindRawHtml"));
        assert!(js.contains("data-sh=\"0\""));
        assert!(js.contains("[md]"));
    }

    #[test]
    fn sail_window_emits_on_against_window() {
        let js = compile(
            "<script>function r(){}</script>\n<template><sail:window on:resize={r} /></template>",
        );
        assert!(js.contains("on(window, \"resize\", r)"));
    }

    #[test]
    fn sail_document_emits_on_against_document() {
        let js = compile(
            "<script>function v(){}</script>\n<template><sail:document on:visibilitychange={v} /></template>",
        );
        assert!(js.contains("on(document, \"visibilitychange\", v)"));
    }

    #[test]
    fn slot_emits_resolveSlots_call() {
        let js = compile(
            "<template><div><slot name=\"header\"><b>fallback</b></slot><slot></slot></div></template>",
        );
        assert!(js.contains("resolveSlots"));
        assert!(js.contains("snapshotChildren(host)"));
        assert!(js.contains("name: \"header\""));
        assert!(js.contains("name: \"default\""));
        assert!(js.contains("data-ss=\"0\""));
        assert!(js.contains("data-ss=\"1\""));
    }

    /// Regression test for gap #21: `{#if}` inside slot fallback content.
    /// The fallback must snapshot `_nodes` AFTER its inner wires run, so
    /// the bindIf anchor (a Comment that replaces the `{#if}` placeholder
    /// at wire-time) ends up in the returned node list. Otherwise
    /// `resolveSlots` splices in stale element refs and the bindIf's
    /// then-branch renders into an orphaned fragment.
    #[test]
    fn slot_fallback_with_if_snapshots_nodes_after_wires() {
        let js = compile(
            "<script>prop show: boolean = false</script>\n\
             <template>\
               <slot name=\"icon\">\
                 {#if show}<b>fallback</b>{/if}\
               </slot>\
             </template>",
        );
        // The fallback's render arrow must declare `_c` BEFORE `_nodes`
        // (so wires populate `_c` and mutate `_root` first), then
        // snapshot. Asserts the ordering by string-distance: the `_c`
        // line must come before `_nodes` in the fallback closure.
        let fallback_open = js.find("fallback:").expect("fallback declared");
        let after = &js[fallback_open..];
        let c_idx = after.find("const _c = []").expect("_c declared in fallback");
        let nodes_idx = after
            .find("const _nodes = Array.from(_root.childNodes)")
            .expect("_nodes declared in fallback");
        assert!(
            c_idx < nodes_idx,
            "fallback must declare _c (and run wires) before snapshotting _nodes; \
             see compiler/runtime.rs::emit_slot_decl"
        );
        // Sanity: the bindIf is actually emitted somewhere in the file.
        assert!(js.contains("bindIf("));
    }

    #[test]
    fn data_static_emits_signal() {
        let js = compile(
            "<script>data jobs: Job[] = []</script>\n<template><span>{jobs.length}</span></template>",
        );
        assert!(js.contains("const jobs = signal(([]))"));
        assert!(!js.contains("_d_jobs"));
    }

    #[test]
    fn data_live_emits_data_subscribe() {
        let js = compile(
            "<script>data live mesh = ship.mesh.snapshot()</script>\n<template><span>{mesh.id}</span></template>",
        );
        assert!(js.contains("data({"));
        assert!(js.contains("subscribe:"));
        assert!(js.contains("/_sail/data?expr=ship.mesh.snapshot()"));
        assert!(js.contains("const mesh = _d_mesh.signal"));
        assert!(js.contains("cleanups.push(_d_mesh.dispose)"));
    }

    #[test]
    fn await_block_emits_bindAwait() {
        let js = compile(
            "<script>state p = Promise.resolve(1)</script>\n<template>{#await p}<i>L</i>{:then v}<b>{v}</b>{:catch e}<u>{e}</u>{/await}</template>",
        );
        assert!(js.contains("bindAwait"));
        assert!(js.contains("data-sw=\"0\""));
        assert!(js.contains("promise: () => (p())"));
        assert!(js.contains("loading: () =>"));
        assert!(js.contains("then_: (_v) =>"));
        assert!(js.contains("catch_: (_e) =>"));
        assert!(js.contains("const v = signal(_v)"));
        assert!(js.contains("const e = signal(_e)"));
    }

    #[test]
    fn component_prop_emits_reactive_bindAttr() {
        // <my-btn label={title} count={n}> — both should reactively
        // sync to the child component's attributes.
        let js = compile(
            "<script>state title = 'Hi'\nstate n = 3</script>\n<template><my-btn label={title} count={n}>x</my-btn></template>",
        );
        assert!(js.contains("<my-btn"));
        assert!(js.contains("bindAttr(_n, \"label\""));
        assert!(js.contains("bindAttr(_n, \"count\""));
        assert!(js.contains("[title]"));
        assert!(js.contains("[n]"));
    }

    #[test]
    fn component_camelcase_prop_kebabs() {
        let js = compile(
            "<script>state url = 'x'</script>\n<template><my-typeahead fetchUrl={url} /></template>",
        );
        assert!(js.contains("bindAttr(_n, \"fetch-url\""));
    }

    #[test]
    fn for_with_index_pattern() {
        let js = compile(
            "<script>state xs = []</script>\n<template>{#for x, i of xs}<li>{i}: {x}</li>{/for}</template>",
        );
        assert!(js.contains("const x = signal(_initialItem)"));
        assert!(js.contains("const i = signal(_initialIdx)"));
        assert!(js.contains("update: (nextItem, nextIdx)"));
    }

    #[test]
    fn wrap_reactive_calls_basic() {
        let r = vec!["count".to_string()];
        assert_eq!(wrap_reactive_calls("count", &r), "count()");
        assert_eq!(wrap_reactive_calls("count + 1", &r), "count() + 1");
        assert_eq!(wrap_reactive_calls("String(count)", &r), "String(count())");
        assert_eq!(wrap_reactive_calls("count.length", &r), "count().length");
    }

    #[test]
    fn wrap_skips_already_called_and_set() {
        let r = vec!["count".to_string()];
        assert_eq!(wrap_reactive_calls("count()", &r), "count()");
        assert_eq!(wrap_reactive_calls("count.set(5)", &r), "count.set(5)");
        assert_eq!(
            wrap_reactive_calls("count.set(count() + 1)", &r),
            "count.set(count() + 1)",
        );
    }

    #[test]
    fn wrap_skips_member_rhs() {
        // user.count: only `user` is wrapped if reactive; `count` is a
        // member, never auto-called.
        let r = vec!["user".to_string(), "count".to_string()];
        assert_eq!(wrap_reactive_calls("user.count", &r), "user().count");
    }

    #[test]
    fn wrap_skips_strings() {
        let r = vec!["count".to_string()];
        assert_eq!(
            wrap_reactive_calls("'count is ' + count", &r),
            "'count is ' + count()",
        );
        // `${count}` is a JS expression inside a template literal;
        // reactive idents inside should be auto-called too.
        assert_eq!(
            wrap_reactive_calls("`tpl ${count} text`", &r),
            "`tpl ${count()} text`",
        );
    }

    #[test]
    fn wrap_handles_multiple_idents() {
        let r = vec!["a".to_string(), "b".to_string()];
        assert_eq!(wrap_reactive_calls("a + b * 2", &r), "a() + b() * 2");
    }

    #[test]
    fn form_associated_emits_factory_with_internals_and_schema_flag() {
        let js = compile(
            "<script>component formAssociated\nprop checked: boolean = false</script>\n<template><span>{checked}</span></template>",
        );
        // Schema declares formAssociated.
        assert!(js.contains("formAssociated: true"));
        // Factory takes internals as third param.
        assert!(js.contains("(host, propSignals, internals) => {"));
    }

    #[test]
    fn non_form_associated_omits_internals_and_flag() {
        let js = compile(
            "<script>state x = 0</script>\n<template><span>{x}</span></template>",
        );
        assert!(!js.contains("formAssociated: true"));
        assert!(js.contains("(host, propSignals) => {"));
        assert!(!js.contains(", internals)"));
    }

    #[test]
    fn expr_references_word_boundary() {
        assert!(expr_references("count + 1", "count"));
        assert!(expr_references("x.count", "x"));
        assert!(!expr_references("counter + 1", "count"));
        assert!(!expr_references("'count is 1'", "count"));
        assert!(expr_references("doIt(count)", "count"));
    }
}
