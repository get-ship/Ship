//! Template body parser.
//!
//! Walks a `<template>` body and produces the `Vec<Node>` root tree.
//! Recognised forms:
//!
//! - HTML elements (open/close tags, self-closing, void elements)
//! - Custom-element components (hyphenated tag name)
//! - Text + `{expr}` interpolations
//! - `{#if}` / `{:else if}` / `{:else}` / `{/if}`
//! - `{#for ITEM of LIST [key={EXPR}]}` / `{:empty}` / `{/for}`
//! - `{#await EXPR}` / `{:then VAL}` / `{:catch ERR}` / `{/await}`
//! - `{@html EXPR}`
//! - Attributes: static, interpolated values, `bind:name={...}`,
//!   `on:event[|mod...]={...}`, `slot="x"`, `let:x[=name]`
//!
//! The parser is **expression-shallow**: `{expr}` and `={expr}` capture
//! the inner string verbatim with brace-balance tracking — semantic
//! checks live in the analyzer phase.
//!
//! See `docs/sail/01-language-spec.md` § The `<template>` block.

use anyhow::{Result, anyhow, bail};

use super::ast::{
    Attr, AttrValue, EventBinding, EventModifier, Expr, Node, Pattern, PropBinding, SlotFill,
    Template,
};

pub fn parse(body: &str) -> Result<Template> {
    let mut p = Parser::new(body);
    let root = p.parse_nodes(None)?;
    Ok(Template { root })
}

struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    i: usize,
}

/// Sentinel that ends a node-list parse — see `parse_nodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// `</TAG>` (any tag — caller validates).
    CloseTag,
    /// `{:else}`, `{:else if}`, `{/if}`.
    IfArm,
    /// `{:empty}`, `{/for}`.
    ForArm,
    /// `{:then X}`, `{:catch E}`, `{/await}`.
    AwaitArm,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, bytes: src.as_bytes(), i: 0 }
    }

    fn peek(&self) -> Option<u8> { self.bytes.get(self.i).copied() }
    fn at_end(&self) -> bool { self.i >= self.bytes.len() }

    fn starts_with(&self, s: &str) -> bool {
        self.src[self.i..].starts_with(s)
    }

    fn parse_nodes(&mut self, stop: Option<Stop>) -> Result<Vec<Node>> {
        let mut out: Vec<Node> = Vec::new();
        while !self.at_end() {
            // Block boundary check (close tag / control-flow sentinel).
            if let Some(sentinel) = self.at_stop(stop) {
                if Some(sentinel) == stop || stop.is_none() && false {
                    return Ok(out);
                }
                // For nested control-flow we only stop on the matching
                // sentinel; otherwise the boundary is an error.
                if stop.is_some() {
                    return Ok(out);
                }
                bail!("unexpected control-flow boundary at byte {}", self.i);
            }

            let b = self.peek().unwrap();

            // `</...>` close tag — return to caller (which validates).
            if b == b'<' && self.bytes.get(self.i + 1) == Some(&b'/') {
                if stop == Some(Stop::CloseTag) {
                    return Ok(out);
                }
                bail!("unexpected close tag at byte {}", self.i);
            }

            // `<TAG ...` open
            if b == b'<' && self.is_tag_start(self.i + 1) {
                out.push(self.parse_element()?);
                continue;
            }

            // `<!-- ... -->` HTML comment
            if self.starts_with("<!--") {
                self.i += 4;
                while !self.at_end() && !self.starts_with("-->") {
                    self.i += 1;
                }
                if self.starts_with("-->") {
                    self.i += 3;
                }
                continue;
            }

            // `{#if|{#for|{#await|{@html|{:..|{/..|{expr}`
            if b == b'{' {
                if self.starts_with("{#if") {
                    out.push(self.parse_if()?);
                    continue;
                }
                if self.starts_with("{#for") {
                    out.push(self.parse_for()?);
                    continue;
                }
                if self.starts_with("{#await") {
                    out.push(self.parse_await()?);
                    continue;
                }
                if self.starts_with("{@html") {
                    out.push(self.parse_raw_html()?);
                    continue;
                }
                if self.starts_with("{:") || self.starts_with("{/") {
                    if stop.is_some() {
                        return Ok(out);
                    }
                    bail!("unmatched control-flow tag at byte {}", self.i);
                }
                // plain `{expr}` interpolation
                let expr = self.read_braced_expr()?;
                out.push(Node::Interpolation(Expr { raw: expr }));
                continue;
            }

            // Text run — until next `<` or `{`.
            let start = self.i;
            while let Some(c) = self.peek() {
                if c == b'<' || c == b'{' {
                    break;
                }
                self.i += 1;
            }
            if self.i > start {
                out.push(Node::Text(self.src[start..self.i].to_string()));
            }
        }

        if let Some(s) = stop {
            bail!("unexpected end of template: still expecting {:?}", s);
        }
        Ok(out)
    }

    /// Returns the matched stop sentinel without consuming, if the
    /// current position begins one of the boundary tokens for `stop`.
    fn at_stop(&self, stop: Option<Stop>) -> Option<Stop> {
        match stop {
            Some(Stop::IfArm) => {
                if self.starts_with("{:else") || self.starts_with("{/if}") {
                    Some(Stop::IfArm)
                } else {
                    None
                }
            }
            Some(Stop::ForArm) => {
                if self.starts_with("{:empty}") || self.starts_with("{/for}") {
                    Some(Stop::ForArm)
                } else {
                    None
                }
            }
            Some(Stop::AwaitArm) => {
                if self.starts_with("{:then") || self.starts_with("{:catch") || self.starts_with("{/await}") {
                    Some(Stop::AwaitArm)
                } else {
                    None
                }
            }
            Some(Stop::CloseTag) => {
                if self.bytes.get(self.i) == Some(&b'<') && self.bytes.get(self.i + 1) == Some(&b'/') {
                    Some(Stop::CloseTag)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn is_tag_start(&self, at: usize) -> bool {
        self.bytes.get(at).map(|b| b.is_ascii_alphabetic()).unwrap_or(false)
    }

    // ── Elements ──────────────────────────────────────────────────────

    fn parse_element(&mut self) -> Result<Node> {
        debug_assert_eq!(self.bytes[self.i], b'<');
        self.i += 1;
        let tag_start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'-' || c == b':' || c == b'.' {
                self.i += 1;
            } else {
                break;
            }
        }
        let tag = self.src[tag_start..self.i].to_string();
        if tag.is_empty() {
            bail!("empty tag at byte {}", tag_start);
        }

        let attrs = self.parse_attrs()?;
        // After attrs, expect `/>` or `>`.
        let self_closing = if self.starts_with("/>") {
            self.i += 2;
            true
        } else if self.peek() == Some(b'>') {
            self.i += 1;
            false
        } else {
            bail!("malformed open tag for <{}> at byte {}", tag, self.i);
        };

        let void = is_void_element(&tag);
        let children = if self_closing || void {
            Vec::new()
        } else {
            let kids = self.parse_nodes(Some(Stop::CloseTag))?;
            self.expect_close_tag(&tag)?;
            kids
        };

        Ok(self.classify_element(tag, attrs, children))
    }

    fn expect_close_tag(&mut self, tag: &str) -> Result<()> {
        if !self.starts_with("</") {
            bail!("expected </{}> at byte {}", tag, self.i);
        }
        self.i += 2;
        let start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'-' || c == b':' {
                self.i += 1;
            } else {
                break;
            }
        }
        let got = &self.src[start..self.i];
        if got != tag {
            bail!("mismatched close: expected </{}>, got </{}>", tag, got);
        }
        // skip optional ws then '>'
        while self.peek() == Some(b' ') || self.peek() == Some(b'\t') {
            self.i += 1;
        }
        if self.peek() != Some(b'>') {
            bail!("expected '>' to close </{}> at byte {}", tag, self.i);
        }
        self.i += 1;
        Ok(())
    }

    /// Decide which Node variant the tag corresponds to. The parser's
    /// rule (overridden later by analyzer where it has a registry):
    ///   - `sail:window` → Window
    ///   - `sail:document` → Document
    ///   - tag contains `-` → Component
    ///   - otherwise → Element
    fn classify_element(&self, tag: String, attrs: Vec<Attr>, children: Vec<Node>) -> Node {
        if tag == "sail:window" {
            return Node::Window { events: collect_events(&attrs) };
        }
        if tag == "sail:document" {
            return Node::Document { events: collect_events(&attrs) };
        }
        if tag.contains('-') {
            return Node::Component {
                tag,
                props: collect_props(&attrs),
                events: collect_events(&attrs),
                slots: collect_slot_fills(&attrs, &children),
                children: filter_non_slot_children(children, &attrs),
            };
        }
        Node::Element { tag, attrs, children }
    }

    // ── Attributes ────────────────────────────────────────────────────

    fn parse_attrs(&mut self) -> Result<Vec<Attr>> {
        let mut out: Vec<Attr> = Vec::new();
        loop {
            self.skip_ws();
            let b = self.peek();
            if b == Some(b'>') || b == None {
                break;
            }
            if b == Some(b'/') && self.bytes.get(self.i + 1) == Some(&b'>') {
                break;
            }

            let name_start = self.i;
            while let Some(c) = self.peek() {
                if c == b'=' || c == b'>' || c == b'/' || c.is_ascii_whitespace() {
                    break;
                }
                self.i += 1;
            }
            let name = self.src[name_start..self.i].to_string();
            if name.is_empty() {
                bail!("empty attribute name at byte {}", name_start);
            }

            let value = if self.peek() == Some(b'=') {
                self.i += 1;
                self.parse_attr_value(&name)?
            } else {
                AttrValue::Static(String::new())
            };
            out.push(Attr { name, value });
        }
        Ok(out)
    }

    fn parse_attr_value(&mut self, attr_name: &str) -> Result<AttrValue> {
        // Three forms: "...", '...', or {expr} (no quotes).
        let b = self.peek().ok_or_else(|| anyhow!("missing attr value"))?;

        if b == b'{' {
            // raw `={expr}` form — most common for bindings
            let raw = self.read_braced_expr()?;
            return Ok(self.classify_value(attr_name, AttrPayload::Expr(raw)));
        }

        let quote = match b {
            b'"' | b'\'' => b,
            _ => bail!("attribute value must be quoted or `{{expr}}` at byte {}", self.i),
        };
        self.i += 1;
        let start = self.i;
        let mut interp_count = 0usize;
        while let Some(c) = self.peek() {
            if c == quote {
                break;
            }
            if c == b'{' {
                let _ = self.read_braced_expr()?;
                interp_count += 1;
                continue;
            }
            self.i += 1;
        }
        if self.peek() != Some(quote) {
            bail!("unterminated attribute value at byte {}", start);
        }
        let raw = self.src[start..self.i].to_string();
        self.i += 1; // closing quote

        if interp_count == 0 {
            Ok(self.classify_value(attr_name, AttrPayload::Static(raw)))
        } else {
            // Interpolated string literal → wrap as an Expr.
            Ok(self.classify_value(attr_name, AttrPayload::Interpolated(raw)))
        }
    }

    fn classify_value(&self, name: &str, payload: AttrPayload) -> AttrValue {
        if let Some(rest) = name.strip_prefix("bind:") {
            let _ = rest;
            return AttrValue::BindTwoWay(Expr { raw: payload.into_expr() });
        }
        if let Some(spec) = name.strip_prefix("on:") {
            let (event, modifiers) = split_event_spec(spec);
            return AttrValue::OnEvent {
                event,
                modifiers,
                handler: Expr { raw: payload.into_expr() },
            };
        }
        match payload {
            AttrPayload::Static(s) => AttrValue::Static(s),
            AttrPayload::Expr(e) => AttrValue::Interpolated(Expr { raw: e }),
            AttrPayload::Interpolated(s) => AttrValue::Interpolated(Expr { raw: s }),
        }
    }

    // ── Control flow ──────────────────────────────────────────────────

    fn parse_if(&mut self) -> Result<Node> {
        // `{#if EXPR}` body `{:else if EXPR2}` body … `{:else}` body `{/if}`
        debug_assert!(self.starts_with("{#if"));
        self.i += 4;
        let cond = self.read_until_close_brace()?;
        let then_ = self.parse_nodes(Some(Stop::IfArm))?;

        let mut elifs: Vec<(Expr, Vec<Node>)> = Vec::new();
        let mut else_: Option<Vec<Node>> = None;

        loop {
            if self.starts_with("{:else if") {
                self.i += "{:else if".len();
                let c = self.read_until_close_brace()?;
                let body = self.parse_nodes(Some(Stop::IfArm))?;
                elifs.push((Expr { raw: c.trim().to_string() }, body));
                continue;
            }
            if self.starts_with("{:else}") {
                self.i += "{:else}".len();
                let body = self.parse_nodes(Some(Stop::IfArm))?;
                else_ = Some(body);
                continue;
            }
            if self.starts_with("{/if}") {
                self.i += "{/if}".len();
                break;
            }
            bail!("unexpected token in `if` block at byte {}", self.i);
        }
        Ok(Node::If { cond: Expr { raw: cond.trim().to_string() }, then_, elifs, else_ })
    }

    fn parse_for(&mut self) -> Result<Node> {
        // `{#for ITEM[, IDX] of LIST [key={EXPR}]}` body `{:empty}` body `{/for}`
        debug_assert!(self.starts_with("{#for"));
        self.i += 5;
        let header = self.read_until_close_brace()?;
        let (item, iter, key) = parse_for_header(&header)?;

        let body = self.parse_nodes(Some(Stop::ForArm))?;
        let empty = if self.starts_with("{:empty}") {
            self.i += "{:empty}".len();
            let nodes = self.parse_nodes(Some(Stop::ForArm))?;
            Some(nodes)
        } else {
            None
        };
        if !self.starts_with("{/for}") {
            bail!("expected {{/for}} at byte {}", self.i);
        }
        self.i += "{/for}".len();
        Ok(Node::For { item, iter, key, body, empty })
    }

    fn parse_await(&mut self) -> Result<Node> {
        debug_assert!(self.starts_with("{#await"));
        self.i += 7;
        let promise = self.read_until_close_brace()?;
        let loading = self.parse_nodes(Some(Stop::AwaitArm))?;

        let mut then_: Option<(Pattern, Vec<Node>)> = None;
        let mut catch: Option<(Pattern, Vec<Node>)> = None;

        loop {
            if self.starts_with("{:then") {
                self.i += "{:then".len();
                let pat = self.read_until_close_brace()?;
                let body = self.parse_nodes(Some(Stop::AwaitArm))?;
                then_ = Some((Pattern::Identifier(pat.trim().to_string()), body));
                continue;
            }
            if self.starts_with("{:catch") {
                self.i += "{:catch".len();
                let pat = self.read_until_close_brace()?;
                let body = self.parse_nodes(Some(Stop::AwaitArm))?;
                catch = Some((Pattern::Identifier(pat.trim().to_string()), body));
                continue;
            }
            if self.starts_with("{/await}") {
                self.i += "{/await}".len();
                break;
            }
            bail!("unexpected token in `await` block at byte {}", self.i);
        }
        Ok(Node::Await {
            promise: Expr { raw: promise.trim().to_string() },
            loading,
            then_,
            catch,
        })
    }

    fn parse_raw_html(&mut self) -> Result<Node> {
        debug_assert!(self.starts_with("{@html"));
        self.i += "{@html".len();
        let expr = self.read_until_close_brace()?;
        Ok(Node::RawHtml(Expr { raw: expr.trim().to_string() }))
    }

    // ── Low-level scanners ────────────────────────────────────────────

    /// Read a `{expr}` interpolation including the braces; return inner.
    fn read_braced_expr(&mut self) -> Result<String> {
        debug_assert_eq!(self.bytes[self.i], b'{');
        let start = self.i + 1;
        let mut depth = 1i32;
        let mut j = start;
        let mut in_str: Option<u8> = None;
        while j < self.bytes.len() {
            let b = self.bytes[j];
            if let Some(q) = in_str {
                if b == b'\\' && j + 1 < self.bytes.len() {
                    j += 2;
                    continue;
                }
                if b == q {
                    in_str = None;
                }
                j += 1;
                continue;
            }
            if b == b'"' || b == b'\'' || b == b'`' {
                in_str = Some(b);
                j += 1;
                continue;
            }
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    let raw = self.src[start..j].to_string();
                    self.i = j + 1;
                    return Ok(raw);
                }
            }
            j += 1;
        }
        bail!("unterminated `{{expr}}` starting at byte {}", start - 1)
    }

    /// Like read_braced_expr but the leading `{` was already consumed
    /// by a control-flow keyword like `{#if`. Reads inner up to `}`.
    fn read_until_close_brace(&mut self) -> Result<String> {
        let start = self.i;
        let mut depth = 1i32;
        let mut j = start;
        let mut in_str: Option<u8> = None;
        while j < self.bytes.len() {
            let b = self.bytes[j];
            if let Some(q) = in_str {
                if b == b'\\' && j + 1 < self.bytes.len() {
                    j += 2;
                    continue;
                }
                if b == q {
                    in_str = None;
                }
                j += 1;
                continue;
            }
            if b == b'"' || b == b'\'' || b == b'`' {
                in_str = Some(b);
                j += 1;
                continue;
            }
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    let raw = self.src[start..j].to_string();
                    self.i = j + 1;
                    return Ok(raw);
                }
            }
            j += 1;
        }
        bail!("unterminated control-flow tag starting at byte {}", start)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.i += 1;
            } else {
                break;
            }
        }
    }
}

// ── Helpers used by classify_element ────────────────────────────────────

enum AttrPayload {
    Static(String),
    Expr(String),
    Interpolated(String),
}
impl AttrPayload {
    fn into_expr(self) -> String {
        match self {
            AttrPayload::Static(s) | AttrPayload::Expr(s) | AttrPayload::Interpolated(s) => s,
        }
    }
}

fn collect_events(attrs: &[Attr]) -> Vec<EventBinding> {
    attrs
        .iter()
        .filter_map(|a| match &a.value {
            AttrValue::OnEvent { event, modifiers, handler } => Some(EventBinding {
                event: event.clone(),
                modifiers: modifiers.clone(),
                handler: handler.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn collect_props(attrs: &[Attr]) -> Vec<PropBinding> {
    attrs
        .iter()
        .filter_map(|a| {
            if a.name.starts_with("on:") || a.name.starts_with("bind:") || a.name == "slot" {
                return None;
            }
            let raw = match &a.value {
                AttrValue::Static(s) => format!("{:?}", s), // string literal
                AttrValue::Interpolated(e) => e.raw.clone(),
                AttrValue::BindTwoWay(e) => e.raw.clone(),
                AttrValue::OnEvent { .. } => return None,
            };
            Some(PropBinding {
                name: a.name.clone(),
                value: Expr { raw },
            })
        })
        .collect()
}

fn collect_slot_fills(_attrs: &[Attr], children: &[Node]) -> Vec<SlotFill> {
    // Slot fills are children with `slot="X"` attribute (or default
    // slot for the rest). Currently captured as plain Element children;
    // analyzer will lift named ones. v1 of the parser keeps both
    // representations: Component.children retains the originals; the
    // analyzer/lower phase will route by `slot=` attr.
    let _ = children;
    Vec::new()
}

fn filter_non_slot_children(children: Vec<Node>, _attrs: &[Attr]) -> Vec<Node> {
    children
}

fn split_event_spec(spec: &str) -> (String, Vec<EventModifier>) {
    let mut parts = spec.split('|');
    let event = parts.next().unwrap_or("").to_string();
    let mut mods: Vec<EventModifier> = Vec::new();
    for m in parts {
        let (kind, arg) = match m.split_once(':') {
            Some((k, v)) => (k.to_string(), Some(v.to_string())),
            None => (m.to_string(), None),
        };
        mods.push(EventModifier { kind, arg });
    }
    (event, mods)
}

fn parse_for_header(header: &str) -> Result<(Pattern, Expr, Option<Expr>)> {
    // Header forms (after `{#for `):
    //   ITEM of LIST
    //   ITEM, IDX of LIST
    //   ITEM of LIST key={EXPR}
    //   ITEM, IDX of LIST key={EXPR}
    let h = header.trim();

    // Split off ` key={...}` if present (using brace-balance).
    let (left, key) = match find_top_level_key(h) {
        Some((lhs, rhs)) => (lhs, Some(Expr { raw: rhs })),
        None => (h.to_string(), None),
    };

    let of_pos = find_of(&left)
        .ok_or_else(|| anyhow!("missing `of` in `{{#for}}` header: {:?}", header))?;
    let item_part = left[..of_pos].trim().to_string();
    let iter = left[(of_pos + 2)..].trim().to_string();

    let item = if let Some((a, b)) = item_part.split_once(',') {
        Pattern::Tuple(vec![a.trim().to_string(), b.trim().to_string()])
    } else {
        Pattern::Identifier(item_part)
    };
    Ok((item, Expr { raw: iter }, key))
}

fn find_top_level_key(h: &str) -> Option<(String, String)> {
    // Look for `key=` at top-level (not inside braces).
    let bytes = h.as_bytes();
    let mut i = 0usize;
    let mut depth = 0i32;
    while i + 4 <= bytes.len() {
        let c = bytes[i];
        match c {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth == 0 && i + 4 <= bytes.len() && &h[i..i + 4] == "key=" {
            // Only accept if preceded by whitespace (not part of an
            // identifier like `monkey=`).
            if i == 0 || bytes[i - 1].is_ascii_whitespace() {
                let lhs = h[..i].trim_end().to_string();
                let rhs_start = i + 4;
                if bytes.get(rhs_start) == Some(&b'{') {
                    // braced expression; capture inside
                    let inner = capture_braced(&h[rhs_start..])?;
                    return Some((lhs, inner));
                }
                return Some((lhs, h[rhs_start..].trim().to_string()));
            }
        }
        i += 1;
    }
    None
}

fn capture_braced(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    for (i, &b) in bytes.iter().enumerate() {
        if let Some(q) = in_str {
            if b == q {
                in_str = None;
            }
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            in_str = Some(b);
            continue;
        }
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(s[1..i].to_string());
            }
        }
    }
    None
}

fn find_of(h: &str) -> Option<usize> {
    // Find ` of ` at top-level (not inside braces or strings).
    let bytes = h.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            in_str = Some(b);
            i += 1;
            continue;
        }
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
        }
        if depth == 0 && b == b' ' && &h[i..(i + 4).min(h.len())] == " of " {
            return Some(i + 1);
        }
        i += 1;
    }
    None
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(s: &str) -> Vec<Node> {
        parse(s).unwrap().root
    }

    #[test]
    fn plain_text() {
        let n = parse_str("hello world");
        assert_eq!(n.len(), 1);
        match &n[0] {
            Node::Text(t) => assert_eq!(t, "hello world"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn interpolation() {
        let n = parse_str("hello {name}!");
        assert_eq!(n.len(), 3);
        assert!(matches!(&n[1], Node::Interpolation(e) if e.raw == "name"));
    }

    #[test]
    fn nested_braces_in_interpolation() {
        let n = parse_str("{ {a:1, b:{c:2}} }");
        match &n[0] {
            Node::Interpolation(e) => assert!(e.raw.contains("c:2")),
            _ => panic!("expected interpolation"),
        }
    }

    #[test]
    fn simple_element() {
        let n = parse_str("<div>hi</div>");
        match &n[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "div");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn nested_elements() {
        let n = parse_str("<div><p>hi</p></div>");
        match &n[0] {
            Node::Element { children, .. } => match &children[0] {
                Node::Element { tag, .. } => assert_eq!(tag, "p"),
                _ => panic!("expected nested p"),
            },
            _ => panic!("expected div"),
        }
    }

    #[test]
    fn void_element() {
        let n = parse_str("<input>");
        match &n[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "input");
                assert!(children.is_empty());
            }
            _ => panic!("expected input"),
        }
    }

    #[test]
    fn self_closing_component() {
        let n = parse_str("<my-comp prop=\"x\" />");
        match &n[0] {
            Node::Component { tag, props, children, .. } => {
                assert_eq!(tag, "my-comp");
                assert_eq!(props.len(), 1);
                assert!(children.is_empty());
            }
            _ => panic!("expected component"),
        }
    }

    #[test]
    fn component_with_event() {
        let n = parse_str("<my-btn on:click={go}>Hi</my-btn>");
        match &n[0] {
            Node::Component { events, children, .. } => {
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].event, "click");
                assert_eq!(events[0].handler.raw, "go");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected component"),
        }
    }

    #[test]
    fn element_with_static_attr() {
        let n = parse_str("<a href=\"/x\">link</a>");
        match &n[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs[0].name, "href");
                assert!(matches!(&attrs[0].value, AttrValue::Static(s) if s == "/x"));
            }
            _ => panic!("expected anchor"),
        }
    }

    #[test]
    fn element_with_interpolated_attr() {
        let n = parse_str("<a href=\"/p/{id}\">x</a>");
        match &n[0] {
            Node::Element { attrs, .. } => {
                assert!(matches!(&attrs[0].value, AttrValue::Interpolated(_)));
            }
            _ => panic!("expected anchor"),
        }
    }

    #[test]
    fn element_with_braced_expr_attr() {
        let n = parse_str("<button disabled={!ok}>x</button>");
        match &n[0] {
            Node::Element { attrs, .. } => match &attrs[0].value {
                AttrValue::Interpolated(e) => assert_eq!(e.raw, "!ok"),
                _ => panic!("expected interpolated"),
            },
            _ => panic!("expected button"),
        }
    }

    #[test]
    fn bind_value_attr() {
        let n = parse_str("<input bind:value={draft} />");
        match &n[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs[0].name, "bind:value");
                match &attrs[0].value {
                    AttrValue::BindTwoWay(e) => assert_eq!(e.raw, "draft"),
                    _ => panic!("expected bind"),
                }
            }
            _ => panic!("expected input"),
        }
    }

    #[test]
    fn on_with_modifiers() {
        let n = parse_str("<form on:submit|preventDefault|key:Enter={save}></form>");
        match &n[0] {
            Node::Element { attrs, .. } => match &attrs[0].value {
                AttrValue::OnEvent { event, modifiers, handler } => {
                    assert_eq!(event, "submit");
                    assert_eq!(modifiers.len(), 2);
                    assert_eq!(modifiers[0].kind, "preventDefault");
                    assert_eq!(modifiers[1].kind, "key");
                    assert_eq!(modifiers[1].arg.as_deref(), Some("Enter"));
                    assert_eq!(handler.raw, "save");
                }
                _ => panic!("expected on:event"),
            },
            _ => panic!("expected form"),
        }
    }

    #[test]
    fn if_basic() {
        let n = parse_str("{#if x}<p>Y</p>{/if}");
        match &n[0] {
            Node::If { cond, then_, elifs, else_ } => {
                assert_eq!(cond.raw, "x");
                assert_eq!(then_.len(), 1);
                assert!(elifs.is_empty());
                assert!(else_.is_none());
            }
            _ => panic!("expected if"),
        }
    }

    #[test]
    fn if_else_chain() {
        let n = parse_str("{#if a}A{:else if b}B{:else}C{/if}");
        match &n[0] {
            Node::If { elifs, else_, .. } => {
                assert_eq!(elifs.len(), 1);
                assert_eq!(elifs[0].0.raw, "b");
                assert!(else_.is_some());
            }
            _ => panic!("expected if"),
        }
    }

    #[test]
    fn for_basic() {
        let n = parse_str("{#for it of list}<li>{it}</li>{/for}");
        match &n[0] {
            Node::For { item, iter, key, .. } => {
                assert!(matches!(item, Pattern::Identifier(s) if s == "it"));
                assert_eq!(iter.raw, "list");
                assert!(key.is_none());
            }
            _ => panic!("expected for"),
        }
    }

    #[test]
    fn for_with_key_and_index() {
        let n = parse_str("{#for it, i of list key={it.id}}<li>{i}-{it.name}</li>{/for}");
        match &n[0] {
            Node::For { item, key, .. } => {
                assert!(matches!(item, Pattern::Tuple(t) if t == &vec!["it".to_string(), "i".to_string()]));
                assert_eq!(key.as_ref().unwrap().raw, "it.id");
            }
            _ => panic!("expected for"),
        }
    }

    #[test]
    fn for_with_empty() {
        let n = parse_str("{#for x of xs}<li>{x}</li>{:empty}<p>none</p>{/for}");
        match &n[0] {
            Node::For { empty, .. } => assert!(empty.is_some()),
            _ => panic!("expected for"),
        }
    }

    #[test]
    fn await_block() {
        let n = parse_str("{#await p}L{:then v}{v}{:catch e}{e.message}{/await}");
        match &n[0] {
            Node::Await { promise, then_, catch, .. } => {
                assert_eq!(promise.raw, "p");
                assert!(then_.is_some());
                assert!(catch.is_some());
            }
            _ => panic!("expected await"),
        }
    }

    #[test]
    fn raw_html() {
        let n = parse_str("{@html safe}");
        match &n[0] {
            Node::RawHtml(e) => assert_eq!(e.raw, "safe"),
            _ => panic!("expected raw html"),
        }
    }

    #[test]
    fn comment_skipped() {
        let n = parse_str("<!-- skip me --><p>ok</p>");
        assert_eq!(n.len(), 1);
        match &n[0] {
            Node::Element { tag, .. } => assert_eq!(tag, "p"),
            _ => panic!("expected p"),
        }
    }

    #[test]
    fn mismatched_close_tag_errors() {
        let err = parse("<div></span>").unwrap_err();
        assert!(err.to_string().contains("mismatched close"));
    }

    #[test]
    fn unterminated_interpolation_errors() {
        let err = parse("hello {name").unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn unterminated_for_errors() {
        let err = parse("{#for x of xs}<li>{x}</li>").unwrap_err();
        assert!(err.to_string().contains("expecting") || err.to_string().contains("expected"));
    }

    #[test]
    fn window_special() {
        let n = parse_str("<sail:window on:resize={onR} />");
        match &n[0] {
            Node::Window { events } => {
                assert_eq!(events[0].event, "resize");
            }
            _ => panic!("expected window"),
        }
    }

    #[test]
    fn nested_control_flow() {
        let n = parse_str("{#if a}{#for x of xs}<i>{x}</i>{/for}{/if}");
        match &n[0] {
            Node::If { then_, .. } => match &then_[0] {
                Node::For { iter, .. } => assert_eq!(iter.raw, "xs"),
                _ => panic!("expected nested for"),
            },
            _ => panic!("expected if"),
        }
    }

    #[test]
    fn quoted_attr_with_braces_treats_as_interpolated() {
        // class="btn {variant}" — the {variant} inside the value
        let n = parse_str("<button class=\"btn {variant}\"></button>");
        match &n[0] {
            Node::Element { attrs, .. } => {
                assert!(matches!(&attrs[0].value, AttrValue::Interpolated(_)));
            }
            _ => panic!("expected button"),
        }
    }
}
