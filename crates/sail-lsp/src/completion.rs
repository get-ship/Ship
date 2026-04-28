//! `textDocument/completion` — context-driven dispatch on the cursor's
//! `<script>` / `<template>` section and the textual line prefix.
//! Triggers: `<` `{` `.` `:` `=`.

use lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind, Position,
};
use sail_compiler::analyze::symbols::{SignalKind, SymbolTable};
use sail_compiler::parse::{parser as sail_parser, script as sail_script};

use crate::manifest::{framework, ship_ops, ShipOp};

pub fn completions(text: &str, pos: Position) -> Vec<CompletionItem> {
    let symbols = permissive_symbols(text);
    let prefix = line_prefix(text, pos);
    if let Some(items) = ship_path_completions(prefix) { return items; }
    match section_at(text, byte_offset(text, pos)) {
        Section::Script if at_line_start(prefix) => directive_completions(),
        Section::Script => signal_completions(&symbols),
        Section::Template => template_context(prefix, &symbols),
        Section::None => Vec::new(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section { Script, Template, None }

fn template_context(prefix: &str, symbols: &SymbolTable) -> Vec<CompletionItem> {
    if let Some(v) = bind_completions(prefix) { return v; }
    if let Some(v) = on_completions(prefix) { return v; }
    if let Some(v) = component_tag_completions(prefix) { return v; }
    if let Some(v) = component_prop_completions(prefix) { return v; }
    if inside_interpolation(prefix) { return signal_completions(symbols); }
    Vec::new()
}

fn directive_completions() -> Vec<CompletionItem> {
    let fw = framework();
    let mut out: Vec<CompletionItem> = fw.directives.iter().map(|d| {
        let label = if d.name == "component_form_associated" { "component formAssociated".into() } else { d.name.clone() };
        item(label, CompletionItemKind::KEYWORD, &d.syntax, &d.description)
    }).collect();
    out.extend(fw.lifecycle_hooks.iter().map(|h| {
        item(h.name.clone(), CompletionItemKind::KEYWORD, &h.syntax,
             &format!("{}\n\n*fires*: {}", h.description, h.fires))
    }));
    out
}

fn ship_path_completions(prefix: &str) -> Option<Vec<CompletionItem>> {
    let rest = trailing_dotted(prefix).strip_prefix("ship.")?;
    let parts: Vec<&str> = rest.split('.').collect();
    let ops = ship_ops();
    Some(match parts.as_slice() {
        [_] => ops.entries.values().map(ship_op_item).collect(),
        [domain, _] => ops.entries.values().filter(|o| o.domain() == *domain).map(ship_op_item).collect(),
        _ => return None,
    })
}

fn ship_op_item(op: &ShipOp) -> CompletionItem {
    CompletionItem {
        label: op.name.clone(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some(op.signature()),
        documentation: Some(markdown(op.markdown())),
        ..Default::default()
    }
}

fn bind_completions(prefix: &str) -> Option<Vec<CompletionItem>> {
    if !is_ident(trailing_after("bind:", prefix)?) { return None; }
    Some(["value", "checked"].iter().map(|n| item(
        (*n).into(), CompletionItemKind::PROPERTY,
        &format!("bind:{n}"),
        &format!("Two-way binding for the `{n}` property of the input."),
    )).collect())
}

fn on_completions(prefix: &str) -> Option<Vec<CompletionItem>> {
    if !is_ident(trailing_after("on:", prefix)?) { return None; }
    Some(["click","input","change","keydown","submit","focus","blur"].iter().map(|n| CompletionItem {
        label: (*n).into(),
        kind: Some(CompletionItemKind::EVENT),
        detail: Some(format!("on{n} event")),
        ..Default::default()
    }).collect())
}

/// Tags pulled from the framework manifest's `components` catalog.
/// Triggered after `<` in template position when the partially-typed
/// tag is still a valid kebab identifier.
fn component_tag_completions(prefix: &str) -> Option<Vec<CompletionItem>> {
    let t = trailing_after_lt(prefix)?;
    if !is_kebab(t) { return None; }
    let comps = &framework().components;
    if comps.is_empty() { return None; }
    Some(comps.iter().map(|c| CompletionItem {
        label: c.tag.clone(),
        kind: Some(CompletionItemKind::MODULE),
        detail: Some("custom element".into()),
        documentation: Some(markdown(c.description.clone())),
        ..Default::default()
    }).collect())
}

/// Prop completion inside an open custom-element tag. Triggered when
/// the cursor sits at an attribute boundary inside `<ship-foo …`,
/// e.g. `<ship-button |>`, `<ship-button tone="primary" |>`. Returns
/// the per-component prop set from the manifest, kebab-cased to
/// match attribute syntax.
fn component_prop_completions(prefix: &str) -> Option<Vec<CompletionItem>> {
    let (tag, partial) = open_tag_attr_context(prefix)?;
    let comp = framework().components.iter().find(|c| c.tag == tag)?;
    if !partial.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    Some(comp.props.iter().map(|p| {
        let attr = kebab_of(&p.name);
        let detail = format!("{}: {}", attr, p.ty);
        CompletionItem {
            label: attr,
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail),
            documentation: Some(markdown(format!(
                "`{}` prop of `<{}>` — type `{}`{}.",
                p.name,
                tag,
                p.ty,
                if p.required { ", required" } else { "" }
            ))),
            ..Default::default()
        }
    }).collect())
}

/// If `prefix` ends inside an open custom-element start tag (between
/// the tag name and the closing `>` / `/>`), return `(tag, partial)`
/// where `partial` is the token currently being typed. Returns `None`
/// when the cursor is outside a tag, inside `</`, or the tag is not
/// kebab-cased.
fn open_tag_attr_context(prefix: &str) -> Option<(String, &str)> {
    let lt = prefix.rfind('<')?;
    let inside = &prefix[lt + 1..];
    if inside.starts_with('/') { return None; }
    if inside.contains('>') || inside.contains('<') { return None; }
    let mut chars = inside.char_indices();
    let mut name_end = 0;
    for (i, c) in &mut chars {
        if c.is_ascii_alphanumeric() || c == '-' {
            name_end = i + c.len_utf8();
        } else {
            break;
        }
    }
    let tag = &inside[..name_end];
    if !tag.contains('-') { return None; }
    let after_tag = &inside[name_end..];
    if !after_tag.starts_with(|c: char| c.is_ascii_whitespace()) { return None; }
    let partial_start = after_tag
        .rfind(|c: char| c.is_ascii_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    let partial = &after_tag[partial_start..];
    if partial.contains('=') || partial.contains('"') || partial.contains('\'') {
        return None;
    }
    Some((tag.to_string(), partial))
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

fn signal_completions(symbols: &SymbolTable) -> Vec<CompletionItem> {
    let mut out = Vec::new();
    for (name, kind) in &symbols.signals {
        let (label, detail) = (signal_kind_label(kind), signal_detail(kind));
        out.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(detail),
            insert_text: Some(format!("{name}()")),
            documentation: Some(markdown(format!("`{label}` signal — call as `{name}()` to read."))),
            ..Default::default()
        });
    }
    for (name, info) in &symbols.functions {
        out.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(first_line(&info.raw)),
            ..Default::default()
        });
    }
    out
}

pub(crate) fn signal_kind_label(kind: &SignalKind) -> &'static str {
    match kind {
        SignalKind::Prop { .. } => "prop",
        SignalKind::State { .. } => "state",
        SignalKind::Derived => "derived",
        SignalKind::Data { .. } => "data",
    }
}

fn signal_detail(kind: &SignalKind) -> String {
    let ty = match kind {
        SignalKind::Prop { ty, .. } | SignalKind::State { ty } | SignalKind::Data { ty } => ty.as_deref(),
        SignalKind::Derived => None,
    }.unwrap_or("?");
    let bang = matches!(kind, SignalKind::Prop { required: true, .. }).then_some("!").unwrap_or("");
    format!("{}: {ty}{bang}", signal_kind_label(kind))
}

fn item(label: String, kind: CompletionItemKind, detail: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label,
        kind: Some(kind),
        detail: Some(detail.into()),
        documentation: Some(markdown(doc.into())),
        ..Default::default()
    }
}

fn markdown(value: String) -> Documentation {
    Documentation::MarkupContent(MarkupContent { kind: MarkupKind::Markdown, value })
}

/// Best-effort symbol table — re-implements `parse::parse` minus the
/// template parser so editors get signal/function completions even
/// while the template is mid-edit.
pub(crate) fn permissive_symbols(text: &str) -> SymbolTable {
    let Ok(raw) = sail_parser::extract_blocks(text) else { return SymbolTable::default(); };
    let mut file = sail_parser::build_shell(text, raw);
    if let Some(script) = file.script.as_mut() { sail_script::scan(script); }
    SymbolTable::from_file(&file)
}

pub(crate) fn byte_offset(text: &str, pos: Position) -> usize {
    let mut byte = 0usize;
    for (i, line) in text.split('\n').enumerate() {
        if i == pos.line as usize {
            return byte + nth_char_byte(line, pos.character as usize);
        }
        byte += line.len() + 1;
    }
    text.len()
}

pub(crate) fn line_prefix(text: &str, pos: Position) -> &str {
    let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
    &line[..nth_char_byte(line, pos.character as usize)]
}

fn nth_char_byte(line: &str, col: usize) -> usize {
    line.char_indices().nth(col).map(|(o, _)| o).unwrap_or(line.len())
}

pub(crate) fn section_at(text: &str, byte: usize) -> Section {
    let p = &text[..byte.min(text.len())];
    let win = |o: Option<usize>, c: Option<usize>| matches!((o, c), (Some(o), c) if c.map_or(true, |x| o > x));
    if win(p.rfind("<script"), p.rfind("</script")) { Section::Script }
    else if win(p.rfind("<template"), p.rfind("</template")) { Section::Template }
    else { Section::None }
}

fn at_line_start(prefix: &str) -> bool {
    prefix.trim_start().chars().all(|c| c.is_ascii_alphabetic() || c == '_')
}

fn inside_interpolation(prefix: &str) -> bool {
    let mut d: i32 = 0;
    for ch in prefix.chars() { if ch == '{' { d += 1; } else if ch == '}' { d -= 1; } }
    d > 0
}

fn trailing_dotted(prefix: &str) -> &str {
    let b = prefix.as_bytes();
    let mut i = b.len();
    while i > 0 {
        let c = b[i - 1];
        if c != b'.' && c != b'_' && !c.is_ascii_alphanumeric() { break; }
        i -= 1;
    }
    &prefix[i..]
}

fn trailing_after<'a>(marker: &str, prefix: &'a str) -> Option<&'a str> {
    Some(&prefix[prefix.rfind(marker)? + marker.len()..])
}

fn trailing_after_lt(prefix: &str) -> Option<&str> {
    let idx = prefix.rfind('<')?;
    if prefix[idx..].starts_with("</") { return None; }
    Some(&prefix[idx + 1..])
}

fn is_ident(s: &str) -> bool { s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') }
fn is_kebab(s: &str) -> bool { s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') }

fn first_line(s: &str) -> String { s.split('\n').next().unwrap_or("").trim_end().into() }

#[cfg(test)]
mod tests {
    use super::*;
    fn pos(l: u32, c: u32) -> Position { Position { line: l, character: c } }
    fn labels(items: &[CompletionItem]) -> Vec<&str> { items.iter().map(|i| i.label.as_str()).collect() }

    #[test]
    fn script_line_start_returns_directives_and_hooks() {
        let src = "<script>\n\n</script>\n<template>x</template>";
        let items = completions(src, pos(1, 0));
        let l = labels(&items);
        for w in ["prop","state","derived","data","function","event","mount","effect","component formAssociated"] {
            assert!(l.contains(&w), "missing {w} in {l:?}");
        }
    }

    #[test]
    fn interpolation_returns_local_signals() {
        let src = "<script>\nstate count = 0\nstate name = ''\n</script>\n<template>{</template>";
        let items = completions(src, pos(4, 11));
        let l = labels(&items);
        assert!(l.contains(&"count") && l.contains(&"name"), "{l:?}");
    }

    #[test]
    fn ship_dot_path_returns_ship_ops() {
        let src = "<script>\ndata x = ship.\n</script>\n<template>x</template>";
        assert!(completions(src, pos(1, 14)).iter().any(|i| i.label.starts_with("agents.")));
        let src2 = "<script>\ndata x = ship.agents.\n</script>\n<template>x</template>";
        let items = completions(src2, pos(1, 21));
        assert!(!items.is_empty());
        assert!(items.iter().all(|i| i.label.starts_with("agents.")));
    }

    #[test]
    fn after_bind_returns_value_and_checked() {
        let src = "<script></script>\n<template><input bind:></template>";
        let items = completions(src, pos(1, 22));
        let l = labels(&items);
        assert!(l.contains(&"value") && l.contains(&"checked"), "{l:?}");
    }

    #[test]
    fn unknown_context_returns_empty() {
        assert!(completions("", pos(0, 0)).is_empty());
    }

    #[test]
    fn after_lt_returns_component_tags() {
        let src = "<script></script>\n<template><</template>";
        let items = completions(src, pos(1, 11));
        let l = labels(&items);
        for required in ["ship-button", "ship-card", "ship-input"] {
            assert!(l.contains(&required), "missing {required} in {l:?}");
        }
    }

    #[test]
    fn inside_component_tag_returns_props() {
        let src = "<script></script>\n<template><ship-button ></ship-button></template>";
        let items = completions(src, pos(1, 23));
        let l = labels(&items);
        for required in ["tone", "variant", "size", "disabled"] {
            assert!(l.contains(&required), "missing {required} in {l:?}");
        }
        assert!(items.iter().all(|i| i.kind == Some(CompletionItemKind::PROPERTY)));
    }
}
