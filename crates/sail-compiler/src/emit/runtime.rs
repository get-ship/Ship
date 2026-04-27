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
                self.wires.push(format!(
                    "  {{\n    const _n = {root}.querySelector('[data-st=\"{id}\"]');\n    if (_n) {cl}.push(bindText(_n, () => String({}), {deps}));\n  }}",
                    expr.raw
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
                self.wires.push(format!(
                    "  {{\n    const _n = {root}.querySelector('[data-sh=\"{id}\"]');\n    if (_n) {cl}.push(bindRawHtml(_n, () => ({expr}), {deps}));\n  }}",
                    expr = expr.raw,
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
                    self.wires.push(format!(
                        "  {{\n    const _n = {root}.querySelector('[data-sa=\"{id}\"]');\n    if (_n) {cl}.push(bindAttr(_n, {name:?}, () => ({expr}), {deps}));\n  }}",
                        name = a.name,
                        expr = expr.raw,
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
                    self.wires.push(format!(
                        "  {{\n    const _n = {root}.querySelector('[data-sa=\"{id}\"]');\n    if (_n) {cl}.push(on(_n, {event:?}, {handler}{opts}));\n  }}",
                        handler = handler.raw,
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
        let saved_sa = self.next_anchor;
        let saved_st = self.next_text_anchor;
        let saved_root = std::mem::replace(&mut self.query_root, "_root".into());
        let saved_cl = std::mem::replace(&mut self.cleanup_array, "_c".into());
        self.next_anchor = 0;
        self.next_text_anchor = 0;
        for s in extra_scope { self.reactive.push(s.clone()); }

        for n in nodes { self.emit_node(n); }

        let captured_html = std::mem::replace(&mut self.html, saved_html);
        let captured_wires = std::mem::replace(&mut self.wires, saved_wires);
        self.next_anchor = saved_sa;
        self.next_text_anchor = saved_st;
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
        out.push_str("      const _nodes = Array.from(_root.childNodes)\n");
        out.push_str("      const _c = []\n");
        for w in wires {
            // Re-indent each captured wire to fit inside the closure.
            for line in w.lines() {
                out.push_str("      ");
                out.push_str(line.trim_start());
                out.push('\n');
            }
        }
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
        wire.push_str(&format!(
            "    {cl}.push(bindIf({{\n      anchor: _anchor,\n      cond: () => ({cond}),\n      then_: {then_arrow}",
            cond = cond.raw,
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

        // The outer arrow returns a single anchor + an internal bindIf
        // that drives that anchor. cleanup teardown disposes the inner.
        let mut arrow = String::from("() => {\n");
        arrow.push_str("      const _anchor = document.createComment('si-elif')\n");
        arrow.push_str("      const _frag = document.createDocumentFragment()\n");
        arrow.push_str("      _frag.appendChild(_anchor)\n");
        arrow.push_str(&format!(
            "      const _d = bindIf({{ anchor: _anchor, cond: () => ({cond}), then_: {then_}",
            cond = head_cond.raw,
            then_ = inner_then_arrow,
        ));
        if let Some(e) = inner_else {
            arrow.push_str(&format!(", else_: {e}"));
        }
        arrow.push_str(" })\n");
        arrow.push_str("      return { nodes: [_anchor], cleanup: _d }\n");
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
            out.push_str("      const _nodes = Array.from(_root.childNodes)\n");
            out.push_str("      const _c = []\n");
            for line in w.iter().flat_map(|s| s.lines()) {
                out.push_str("      ");
                out.push_str(line.trim_start());
                out.push('\n');
            }
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
            wire.push_str(&format!(
                "    {cl}.push(on({target}, {ev:?}, {h}{opts}));\n",
                ev = e.event,
                h = e.handler.raw,
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
        wire.push_str(&format!(
            "    {cl}.push(bindAwait({{\n      anchor: _anchor,\n      promise: () => ({p}),\n      loading: {loading}",
            p = promise.raw,
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

        let key_expr = match key {
            Some(k) => format!(
                "({item}, _i) => ({k})",
                item = item_name,
                k = k.raw,
            ),
            None => "(_, i) => i".to_string(),
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
                "    {cl}.push(bindIf({{ anchor: _eAnchor, cond: () => ({iter}).length === 0, then_: {empty} }}))\n",
                iter = iter.raw,
                empty = empty,
            ));
        } else {
            wire.push_str("    if (_slot) _slot.replaceWith(_anchor)\n");
        }
        wire.push_str(&format!(
            "    {cl}.push(bindFor({{\n      anchor: _anchor,\n      items: () => ({iter}),\n      key: {key},\n      render: {body}\n    }}))\n",
            iter = iter.raw,
            key = key_expr,
            body = body_arrow,
        ));
        wire.push_str("  }");
        self.wires.push(wire);
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
            // `data live` adds its own `data` import via self.imports
            // during script emit; nothing extra needed here.
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

        // Factory.
        out.push_str(&format!(
            "defineComponent({tag:?}, SCHEMA, (host, propSignals) => {{\n",
            tag = self.tag
        ));
        out.push_str("  const cleanups = []\n");

        if let Some(s) = script {
            // Props: pull from propSignals + expose as const for script body.
            for p in &s.props {
                let initial = p.default.as_deref().unwrap_or("undefined");
                out.push_str(&format!(
                    "  const {name} = propSignals.{name} = signal({initial})\n",
                    name = p.name
                ));
            }
            // State & derived.
            for st in &s.state {
                out.push_str(&format!(
                    "  const {name} = signal({initial})\n",
                    name = st.name,
                    initial = st.initial
                ));
            }
            for d in &s.derived {
                out.push_str(&format!(
                    "  const {name} = derived(() => ({expr}))\n",
                    name = d.name,
                    expr = d.expr
                ));
            }
            // `data X = expr`        → signal seeded by expr (static; analyzer
            //                          phase can swap to a server-resolved value)
            // `data live X = expr`   → data({ initial, subscribe }) where
            //                          subscribe URL falls back to the expr
            //                          source string (analyzer rewrites once
            //                          schema sources resolve it)
            for d in &s.data {
                if d.live {
                    self.imports.insert("data");
                    out.push_str(&format!(
                        "  const _d_{name} = data({{ initial: ({expr}), subscribe: {url:?} }})\n  const {name} = _d_{name}.signal\n  cleanups.push(_d_{name}.dispose)\n",
                        name = d.name,
                        expr = d.expr,
                        url = format!("/_sail/data?expr={}", d.expr),
                    ));
                } else {
                    out.push_str(&format!(
                        "  const {name} = signal(({expr}))\n",
                        name = d.name,
                        expr = d.expr,
                    ));
                }
            }
            // User functions verbatim.
            for f in &s.functions {
                out.push_str("  ");
                out.push_str(&f.raw.replace('\n', "\n  "));
                out.push('\n');
            }
        }

        // Slots: snapshot children BEFORE innerHTML so light-DOM fills survive.
        if !self.slots.is_empty() {
            self.imports.insert("snapshotChildren" as &'static str);
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
    if s.props.is_empty() {
        return "{ observedAttrs: [], props: {} }".to_string();
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
    out.push_str(" } }");
    out
}

fn decoder_for(ty: Option<&str>) -> &'static str {
    match ty.map(str::trim) {
        Some("number") => "(r) => r === null || r === '' ? NaN : Number(r)",
        Some("boolean") => "(r) => r !== null && r.toLowerCase() !== 'false'",
        _ => "(r) => r ?? ''",
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
        assert!(js.contains("const d = derived(() => (c * 2))"));
        assert!(js.contains("[d]") || js.contains("[c, d]"));
    }

    #[test]
    fn prop_decl_creates_signal_and_schema() {
        let js = compile(
            "<script>prop label: string = 'hi'</script>\n<template><span>{label}</span></template>",
        );
        assert!(js.contains("propSignals.label = signal('hi')"));
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
        assert!(js.contains("cond: () => (on)"));
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
        assert!(js.contains("items: () => (list)"));
        assert!(js.contains("key: (it, _i) => (it.id)"));
        assert!(js.contains("const it = signal(_initialItem)"));
        assert!(js.contains("update: (nextItem, nextIdx)"));
        // Inner bindText reads it() and tracks `it` as a dep.
        assert!(js.contains("String(it.name)"));
        assert!(js.contains("[it]"));
    }

    #[test]
    fn for_with_empty_emits_bindIf_for_empty_branch() {
        let js = compile(
            "<script>state xs = []</script>\n<template>{#for x of xs}<li>{x}</li>{:empty}<p>none</p>{/for}</template>",
        );
        assert!(js.contains("bindFor"));
        assert!(js.contains("bindIf"));
        assert!(js.contains("cond: () => (xs).length === 0"));
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
        assert!(js.contains("promise: () => (p)"));
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
    fn expr_references_word_boundary() {
        assert!(expr_references("count + 1", "count"));
        assert!(expr_references("x.count", "x"));
        assert!(!expr_references("counter + 1", "count"));
        assert!(!expr_references("'count is 1'", "count"));
        assert!(expr_references("doIt(count)", "count"));
    }
}
