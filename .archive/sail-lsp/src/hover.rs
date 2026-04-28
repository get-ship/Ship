//! `textDocument/hover` — resolves the identifier (or dotted path)
//! under the cursor against the framework manifest, ship-ops manifest,
//! and the document's local symbol table.

use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};
use sail_compiler::analyze::symbols::{FunctionInfo, SignalKind};

use crate::completion::{byte_offset, permissive_symbols, section_at, signal_kind_label, Section};
use crate::manifest::{framework, ship_ops};

pub fn hover(text: &str, pos: Position) -> Option<Hover> {
    let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
    let col = pos.character as usize;
    if let Some(h) = template_directive_hover(line, col, pos.line) { return Some(h); }
    let (word_range, word) = word_at(line, col, pos.line)?;
    if let Some(h) = dotted_path_hover(line, col, pos.line) { return Some(h); }
    let symbols = permissive_symbols(text);
    let section = section_at(text, byte_offset(text, pos));
    if section == Section::Script || section == Section::Template {
        if let Some(kind) = symbols.signals.get(word) {
            return Some(markup(signal_markdown(word, kind), word_range));
        }
        if let Some(info) = symbols.functions.get(word) {
            return Some(markup(function_markdown(word, info), word_range));
        }
    }
    framework_keyword_hover(word, word_range)
        .or_else(|| scope_binding_hover(word, word_range))
}

fn dotted_path_hover(line: &str, col: usize, ln: u32) -> Option<Hover> {
    let (start, end, text) = dotted_at(line, col)?;
    let rest = text.strip_prefix("ship.")?;
    let op = ship_ops().entries.get(rest)?;
    Some(markup(op.markdown(), span(ln, start, end)))
}

fn template_directive_hover(line: &str, col: usize, ln: u32) -> Option<Hover> {
    // Recognise `{#if`, `{#for`, `{#await`, `{@html`, `{:else}`, `{:then …}`,
    // `{:catch …}`, `{:empty}`, `{/if}`, `{/for}`, `{/await}`. We anchor on
    // the `{` that precedes the cursor on the same line.
    let bytes = line.as_bytes();
    if col == 0 || col > bytes.len() { return None; }
    let lbrace = line[..col].rfind('{')?;
    let after = &line[lbrace + 1..];
    let (tag_kind, tag_len) = match after.as_bytes().first()? {
        b'#' | b'/' => parse_block_word(after),
        b':' => parse_branch_word(after),
        b'@' => parse_at_word(after),
        _ => return None,
    }?;
    // Cursor must be inside the `{TAG` span.
    if col > lbrace + 1 + tag_len { return None; }
    let entry = framework().template_syntax.iter().find(|t| t.name == tag_kind)?;
    let mut md = format!("**{{`{}`}}** — {}", entry.syntax, entry.description);
    if let Some(ex) = entry.examples.first() {
        md.push_str("\n\n```\n");
        md.push_str(ex);
        md.push_str("\n```");
    }
    Some(markup(md, span(ln, lbrace, lbrace + 1 + tag_len)))
}

fn parse_block_word(after: &str) -> Option<(&'static str, usize)> {
    // after starts with '#' or '/'
    let kw_start = 1;
    let kw_end = after[kw_start..]
        .find(|c: char| !c.is_ascii_alphabetic())
        .map(|n| kw_start + n)
        .unwrap_or(after.len());
    let kw = &after[kw_start..kw_end];
    let name = match kw {
        "if" => "if_block",
        "for" => "for_block",
        "await" => "await_block",
        _ => return None,
    };
    Some((name, kw_end))
}

fn parse_branch_word(after: &str) -> Option<(&'static str, usize)> {
    let kw_end = after[1..]
        .find(|c: char| !c.is_ascii_alphabetic())
        .map(|n| 1 + n)
        .unwrap_or(after.len());
    let kw = &after[1..kw_end];
    // Branches map to their parent block's manifest entry — best signal
    // we have without a per-branch manifest field.
    let name = match kw {
        "else" | "elif" => "if_block",
        "then" | "catch" => "await_block",
        "empty" => "for_block",
        _ => return None,
    };
    Some((name, kw_end))
}

fn parse_at_word(after: &str) -> Option<(&'static str, usize)> {
    let kw_end = after[1..]
        .find(|c: char| !c.is_ascii_alphabetic())
        .map(|n| 1 + n)
        .unwrap_or(after.len());
    let kw = &after[1..kw_end];
    if kw == "html" { Some(("raw_html", kw_end)) } else { None }
}

fn framework_keyword_hover(word: &str, range: Range) -> Option<Hover> {
    let fw = framework();
    if let Some(d) = fw.directives.iter().find(|d| matches_directive(&d.name, word)) {
        let mut md = format!("```\n{}\n```\n\n{}", d.syntax, d.description);
        if let Some(ex) = d.examples.first() { md.push_str(&format!("\n\n```\n{ex}\n```")); }
        return Some(markup(md, range));
    }
    if let Some(h) = fw.lifecycle_hooks.iter().find(|h| h.name == word) {
        let md = format!("```\n{}\n```\n\n{}\n\n*fires*: {}", h.syntax, h.description, h.fires);
        return Some(markup(md, range));
    }
    None
}

fn matches_directive(name: &str, word: &str) -> bool {
    name == word || (name == "component_form_associated" && word == "component")
}

fn scope_binding_hover(word: &str, range: Range) -> Option<Hover> {
    let b = framework().scope_bindings.iter().find(|b| b.name == word)?;
    let md = format!("```\n{}: {}\n```\n\n{}\n\n*available in*: {}",
                     b.name, b.ty, b.description, b.available_in.join(", "));
    Some(markup(md, range))
}

fn signal_markdown(name: &str, kind: &SignalKind) -> String {
    let label = signal_kind_label(kind);
    let ty = match kind {
        SignalKind::Prop { ty, .. }
        | SignalKind::State { ty }
        | SignalKind::Data { ty }
        | SignalKind::Module { ty, .. } => ty.clone(),
        SignalKind::Derived => None,
    }.unwrap_or_else(|| "<inferred>".into());
    let access = if matches!(kind, SignalKind::Module { .. }) {
        format!("Plain JS binding — read/write as `{name}` directly.")
    } else {
        format!("Reactive signal — read as `{name}()`, write as `{name}.set(v)`.")
    };
    format!("**{label}** `{name}: {ty}`\n\n{access}")
}

fn function_markdown(name: &str, info: &FunctionInfo) -> String {
    let first = info.raw.lines().next().unwrap_or("").trim_end();
    format!("```\nfunction {name}\n```\n\n```\n{first}\n```")
}

fn markup(md: String, range: Range) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: md }),
        range: Some(range),
    }
}

fn span(line: u32, start: usize, end: usize) -> Range {
    Range {
        start: Position { line, character: start as u32 },
        end: Position { line, character: end as u32 },
    }
}

fn word_at(line: &str, col: usize, ln: u32) -> Option<(Range, &str)> {
    let b = line.as_bytes();
    let n = b.len();
    if col > n { return None; }
    let mut s = col;
    while s > 0 && is_ident(b[s - 1]) { s -= 1; }
    let mut e = col;
    while e < n && is_ident(b[e]) { e += 1; }
    if s == e { return None; }
    Some((span(ln, s, e), &line[s..e]))
}

fn dotted_at(line: &str, col: usize) -> Option<(usize, usize, &str)> {
    let b = line.as_bytes();
    let n = b.len();
    if col > n { return None; }
    let mut s = col;
    while s > 0 && (is_ident(b[s - 1]) || b[s - 1] == b'.') { s -= 1; }
    let mut e = col;
    while e < n && (is_ident(b[e]) || b[e] == b'.') { e += 1; }
    if s == e { return None; }
    Some((s, e, &line[s..e]))
}

fn is_ident(c: u8) -> bool { c.is_ascii_alphanumeric() || c == b'_' }

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(l: u32, c: u32) -> Position { Position { line: l, character: c } }

    fn body(h: &Hover) -> String {
        match &h.contents {
            HoverContents::Markup(m) => m.value.clone(),
            other => panic!("expected markup, got {other:?}"),
        }
    }

    #[test]
    fn signal_hover_shows_kind_and_type() {
        let src = "<script>\nstate count: number = 0\n</script>\n<template>{count()}</template>";
        let h = hover(src, pos(1, 7)).expect("hover");
        let s = body(&h);
        assert!(s.contains("state") && s.contains("count") && s.contains("number"), "{s}");
    }

    #[test]
    fn function_hover_shows_signature() {
        let src = "<script>\nstate open = false\nfunction close() { open.set(false) }\n</script>\n<template>x</template>";
        let h = hover(src, pos(2, 11)).expect("hover");
        let s = body(&h);
        assert!(s.contains("function close"), "{s}");
    }

    #[test]
    fn ship_op_hover_resolves_full_path() {
        let src = "<script>\ndata x = ship.agents.list()\n</script>\n<template>x</template>";
        // Cursor on `list`.
        let h = hover(src, pos(1, 23)).expect("hover");
        let s = body(&h);
        assert!(s.contains("ship.agents.list"), "{s}");
        let entry = ship_ops().entries.get("agents.list").unwrap();
        assert!(s.contains(&entry.description), "{s}");
    }

    #[test]
    fn keyword_hover_describes_directive() {
        let src = "<script>\nprop value = ''\n</script>\n<template>x</template>";
        let h = hover(src, pos(1, 1)).expect("hover");
        let s = body(&h);
        assert!(s.contains("prop"));
        let dir = framework().directives.iter().find(|d| d.name == "prop").unwrap();
        assert!(s.contains(&dir.description), "{s}");
    }

    #[test]
    fn unknown_position_returns_none() {
        // Whitespace-only line, far past the script section.
        let src = "<script></script>\n<template>   </template>";
        assert!(hover(src, pos(1, 12)).is_none());
    }

    #[test]
    fn template_directive_hover_resolves_block() {
        let src = "<script></script>\n<template>{#if x}…{/if}</template>";
        // Cursor on the `if` of `{#if`.
        let h = hover(src, pos(1, 12)).expect("hover");
        let s = body(&h);
        let entry = framework().template_syntax.iter().find(|t| t.name == "if_block").unwrap();
        assert!(s.contains(&entry.description), "{s}");
    }
}
