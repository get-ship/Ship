//! Generate `crates/sui/compiler/tokens.json` (`cargo xtask gen-tokens`).
//!
//! `tokens.json` is the registry the `sui-css-token` lint reads: any
//! `var(--name)` in a `.sui` `<style>` block must resolve to a name in this
//! file (or a locally-declared custom property). Historically the file was
//! hand-maintained "in sync" with `packages/sui/tokens/src/{tokens,themes}.css`
//! — its own `$comment` admitted the mirror — and drift made the lint lie
//! (zombie tokens it accepted that no theme declares; declared tokens it was
//! blind to). This generator makes the CSS the single source of truth:
//!
//!   * **theme tokens** — every CSS custom-property *declaration* (`--name:`)
//!     in the two canonical CSS files, parsed deterministically. A token in
//!     the old JSON but not in CSS was a zombie; a token in CSS but not in the
//!     JSON was a lint blind spot. Regenerating erases both.
//!   * **channels** — cross-component contract tokens that no CSS file
//!     *declares* (one component sets `--tone`, another reads it; the shell
//!     sets `--ship-rail-w`; a `.sui` reads `var(--z-overlay, 900)` with a
//!     fallback). They have no CSS declaration to derive from, so they live
//!     here as an explicit, reviewable contract embedded in this generator.
//!
//! The lint flattens every string leaf under a non-`$` key, so the exact
//! nesting is for human review only — but the schema shape (top-level keys →
//! nested arrays of names) is preserved so the lint consumes it unchanged.
//!
//! `--check` regenerates in-memory and byte-compares against the committed
//! file, exiting non-zero on drift (the CI gate). Each file is stamped
//! AUTO-GENERATED.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

/// Canonical CSS sources, relative to the repo root. Order is irrelevant —
/// declarations are unioned into a sorted set — but kept stable for clarity.
const CSS_SOURCES: [&str; 2] = [
    "packages/sui/tokens/src/tokens.css",
    "packages/sui/tokens/src/themes.css",
];

/// Output, relative to the repo root.
const OUT_REL: &str = "crates/sui/compiler/tokens.json";

/// Strip `/* … */` comments so a `--name:` mentioned inside a doc comment is
/// not mistaken for a declaration. CSS custom properties never span comments,
/// so a plain two-state scan is exact.
fn strip_comments(css: &str) -> String {
    let bytes = css.as_bytes();
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2; // skip the closing */
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_lowercase()
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'
}

/// Custom-property *declarations* in `css`: a `--ident` token immediately
/// followed (after optional whitespace) by `:`. This is the only CSS shape
/// that introduces a custom property; a `var(--name)` reference is followed by
/// `)` or `,`, never `:`, so references are not collected. Mirrors the
/// declaration scanner in `crates/sui/lint/src/rules/css_token.rs`.
fn collect_declarations(css: &str, out: &mut std::collections::BTreeSet<String>) {
    let css = strip_comments(css);
    let bytes = css.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && is_ident_start(bytes[i + 2]) {
            let name_start = i + 2;
            let mut j = name_start;
            while j < bytes.len() && is_ident_cont(bytes[j]) {
                j += 1;
            }
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if bytes.get(k) == Some(&b':') {
                out.insert(css[name_start..j].to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
}

/// The cross-component contract tokens — no CSS file declares them, so they
/// have no source to derive from and live here as an explicit contract. The
/// `sui-css-token` lint accepts them by membership exactly like declared
/// theme tokens.
fn channels() -> Value {
    json!({
        "$comment": "Cross-component contract tokens — NOT declared in any CSS file. One component sets them and another consumes (a parent sets --tone, a slotted child reads it; the shell sets --ship-rail-w; a .sui reads var(--z-overlay, 900) with a fallback). They have no CSS declaration to derive from, so they live here as an explicit, reviewable contract. The sui-css-token lint flattens these the same as declared theme tokens.",
        "tone-aliases": [
            "tone", "tone-fg", "tone-bg", "tone-border", "tone-ring",
            "tone-contrast", "tone-on-fg"
        ],
        "shell": ["ship-rail-w", "ship-rail-w-collapsed", "ship-top-bar-h"],
        "rail": ["rail-label-display", "rail-item-justify"],
        "anchor-positioning": [
            "sui-popover-anchor", "sui-popover-default",
            "sui-tooltip-anchor", "sui-tooltip-default",
            "sui-hc-anchor", "sui-hc-default",
            "sui-dd-anchor", "sui-dd-default"
        ],
        "toast": ["toast-duration"],
        "z-index": ["z-overlay"],
        "card-flags": ["card-shadow-style"]
    })
}

/// Render the deterministic `tokens.json` content (trailing newline included).
fn render(root: &Path) -> Result<String> {
    let mut tokens = std::collections::BTreeSet::new();
    for rel in CSS_SOURCES {
        let path = root.join(rel);
        let css = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        collect_declarations(&css, &mut tokens);
    }
    let theme_tokens: Vec<Value> = tokens.into_iter().map(Value::String).collect();

    let mut theme = Map::new();
    theme.insert("tokens".to_string(), Value::Array(theme_tokens));

    let mut doc = Map::new();
    doc.insert(
        "$comment".to_string(),
        Value::String(
            "AUTO-GENERATED by `cargo xtask gen-tokens`. Do not edit by hand. \
             `theme.tokens` are the CSS custom-property declarations in \
             packages/sui/tokens/src/{tokens,themes}.css (the single source of \
             truth); `channels` are cross-component contract tokens with no CSS \
             declaration. The sui-css-token lint flattens every leaf. Regenerate \
             with `cargo xtask gen-tokens`; CI gates `gen-tokens --check`."
                .to_string(),
        ),
    );
    doc.insert("theme".to_string(), Value::Object(theme));
    doc.insert("channels".to_string(), channels());

    let mut s = serde_json::to_string_pretty(&Value::Object(doc))?;
    s.push('\n');
    Ok(s)
}

pub fn run(root: &Path, check: bool) -> Result<()> {
    let out = root.join(OUT_REL);
    let content = render(root)?;

    if check {
        let existing = std::fs::read_to_string(&out).unwrap_or_default();
        if existing != content {
            eprintln!("Out of date: {}", out.display());
            eprintln!("Run `cargo xtask gen-tokens` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("tokens.json is up-to-date.");
        return Ok(());
    }

    std::fs::write(&out, &content).with_context(|| format!("writing {}", out.display()))?;
    eprintln!("gen-tokens: wrote {}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_before_scanning() {
        let css = "/* --ghost: 1; */ :root { --real: 2; }";
        let mut set = std::collections::BTreeSet::new();
        collect_declarations(css, &mut set);
        assert!(set.contains("real"));
        assert!(!set.contains("ghost"), "comment-embedded decl must be ignored");
    }

    #[test]
    fn collects_declarations_not_references() {
        let css = ":root { --a: var(--b); height: var(--c); }";
        let mut set = std::collections::BTreeSet::new();
        collect_declarations(css, &mut set);
        // `--a` is declared; `--b` and `--c` are only referenced.
        assert!(set.contains("a"));
        assert!(!set.contains("b"));
        assert!(!set.contains("c"));
    }

    #[test]
    fn declaration_with_whitespace_before_colon() {
        let css = ":root { --foo : 1px; }";
        let mut set = std::collections::BTreeSet::new();
        collect_declarations(css, &mut set);
        assert!(set.contains("foo"));
    }

    #[test]
    fn channels_carry_referenced_contract_tokens() {
        // z-overlay is referenced (with fallback) in the shell launcher but
        // declared in no CSS file — it must survive in the registry via the
        // channels contract, else the lint would regress on that .sui.
        let flat = flatten(&channels());
        for name in ["z-overlay", "tone", "ship-rail-w", "card-shadow-style"] {
            assert!(flat.contains(name), "channels must carry {name}");
        }
    }

    fn flatten(v: &Value) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        walk(v, &mut out);
        out
    }

    fn walk(v: &Value, out: &mut std::collections::BTreeSet<String>) {
        match v {
            Value::String(s) => {
                out.insert(s.clone());
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            Value::Object(m) => {
                for (k, val) in m {
                    if !k.starts_with('$') {
                        walk(val, out);
                    }
                }
            }
            _ => {}
        }
    }
}
