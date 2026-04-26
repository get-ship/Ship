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

use crate::parse::ast::{Attr, AttrValue, EventModifier, Node, ScriptBlock, SailFile, Template};

/// Emit the JS module source for a parsed Sail file. `tag` is the
/// custom-element tag name (typically the file basename, kebab-cased).
pub fn emit(file: &SailFile, tag: &str) -> String {
    let mut e = Emitter::new(tag);
    if let Some(s) = &file.script {
        e.collect_reactive_names(s);
    }
    if let Some(t) = &file.template {
        e.emit_template(t);
    }
    e.render(file.script.as_ref())
}

struct Emitter<'a> {
    tag: &'a str,
    /// Reactive identifier names: state ∪ derived ∪ data ∪ prop names.
    reactive: Vec<String>,
    /// HTML being assembled, with anchor markers.
    html: String,
    /// Wiring instructions to emit inside the factory closure.
    wires: Vec<String>,
    /// Counter for `data-sa` anchors on elements.
    next_anchor: u32,
    /// Counter for `data-st` anchors on text interpolations.
    next_text_anchor: u32,
    /// Imported names from `@sail/runtime`.
    imports: std::collections::BTreeSet<&'static str>,
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
                self.wires.push(format!(
                    "  {{\n    const _n = host.querySelector('[data-st=\"{id}\"]');\n    if (_n) cleanups.push(bindText(_n, () => String({}), {deps}));\n  }}",
                    expr.raw
                ));
            }
            Node::Element { tag, attrs, children } => {
                self.emit_element(tag, attrs, children);
            }
            Node::Component { tag, props, events, children, .. } => {
                // MVP: emit the component tag with prop attrs. Reactive
                // prop bindings degrade to initial-value attributes —
                // a follow-up will wire reactive prop updates.
                let attr_list: Vec<Attr> = props
                    .iter()
                    .map(|p| Attr {
                        name: p.name.clone(),
                        value: AttrValue::Static(p.value.raw.clone()),
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
            Node::If { .. } => self.html.push_str("<!-- sail: {#if} not yet emitted -->"),
            Node::For { .. } => self.html.push_str("<!-- sail: {#for} not yet emitted -->"),
            Node::Await { .. } => self.html.push_str("<!-- sail: {#await} not yet emitted -->"),
            Node::Slot { .. } | Node::SlotFill { .. } => {
                self.html.push_str("<!-- sail: <slot> not yet emitted -->");
            }
            Node::Window { .. } | Node::Document { .. } => {
                self.html.push_str("<!-- sail: <window>/<document> not yet emitted -->");
            }
            Node::RawHtml(_) => self.html.push_str("<!-- sail: {@html} not yet emitted -->"),
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
                    self.wires.push(format!(
                        "  {{\n    const _n = host.querySelector('[data-sa=\"{id}\"]');\n    if (_n) cleanups.push(bindAttr(_n, {name:?}, () => ({expr}), {deps}));\n  }}",
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
                    self.wires.push(format!(
                        "  {{\n    const _n = host.querySelector('[data-sa=\"{id}\"]');\n    if (_n) cleanups.push({bind_fn}(_n, {sig}));\n  }}",
                        sig = expr.raw,
                    ));
                }
                AttrValue::OnEvent { event, modifiers, handler } => {
                    let id = anchor_id.unwrap();
                    self.imports.insert("on");
                    let opts = render_modifier_opts(modifiers);
                    self.wires.push(format!(
                        "  {{\n    const _n = host.querySelector('[data-sa=\"{id}\"]');\n    if (_n) cleanups.push(on(_n, {event:?}, {handler}{opts}));\n  }}",
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

    fn deps_array(&mut self, expr: &str) -> String {
        let used: Vec<&String> = self
            .reactive
            .iter()
            .filter(|name| expr_references(expr, name))
            .collect();
        format!("[{}]", used.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
    }

    fn render(&self, script: Option<&ScriptBlock>) -> String {
        let mut imports: Vec<&'static str> = self.imports.iter().copied().collect();
        // For decoded prop signal seeding.
        if let Some(s) = script {
            if !s.props.is_empty() { imports.push("signal"); }
            if !s.state.is_empty() { imports.push("signal"); }
            if !s.derived.is_empty() { imports.push("derived"); }
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
            // User functions verbatim.
            for f in &s.functions {
                out.push_str("  ");
                out.push_str(&f.raw.replace('\n', "\n  "));
                out.push('\n');
            }
        }

        out.push_str("  host.innerHTML = HTML\n");

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
    fn bind_value_emits_bindValue() {
        let js = compile(
            "<script>state draft = ''</script>\n<template><input bind:value={draft} /></template>",
        );
        assert!(js.contains("bindValue(_n, draft)"));
    }

    #[test]
    fn bind_checked_emits_bindChecked() {
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
    fn expr_references_word_boundary() {
        assert!(expr_references("count + 1", "count"));
        assert!(expr_references("x.count", "x"));
        assert!(!expr_references("counter + 1", "count"));
        assert!(!expr_references("'count is 1'", "count"));
        assert!(expr_references("doIt(count)", "count"));
    }
}
