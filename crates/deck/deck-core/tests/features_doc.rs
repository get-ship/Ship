//! Anti-drift gate for `crates/deck/FEATURES.md`.
//!
//! The feature inventory is only useful if its implementation pointers stay
//! true. This test reads `FEATURES.md`, extracts every implementation
//! reference, and asserts for each that the referenced file exists AND — when
//! the ref carries a `::symbol` suffix — that the file's contents contain that
//! symbol token. A moved or renamed symbol fails the build here, so the doc is
//! a tested artifact rather than prose that rots.
//!
//! Reference grammar (matched anywhere in the file):
//!   - `crates/<...>/<file>.rs::<symbol>` — file must exist, symbol must be present.
//!   - `crates/<...>/<file>.rs`           — file must exist (no symbol check).
//!   - `crates/<...>:<line>`              — file must exist (a `path:line` ref).
//!
//! The repo root is resolved from `CARGO_MANIFEST_DIR`
//! (`<repo>/crates/deck/deck-core`) up three parents, then `FEATURES.md` is read
//! from `<repo>/crates/deck/FEATURES.md`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// `<repo>/crates/deck/deck-core` → `<repo>`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/deck
        .and_then(|p| p.parent()) // crates
        .and_then(|p| p.parent()) // repo root
        .expect("CARGO_MANIFEST_DIR has three ancestors")
        .to_path_buf()
}

/// One extracted reference: the repo-relative file path, and an optional symbol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Ref {
    path: String,
    symbol: Option<String>,
}

/// Characters that can appear in a repo-relative path token we extract. We
/// deliberately stop at backtick, whitespace, `|`, `:`, and `*` (the
/// table-cell and markdown delimiters) so a ref inside a `crates/...` code span
/// is captured cleanly.
fn is_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.')
}

fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Scan `text` for every `crates/...` reference and return the deduped set.
///
/// A reference starts at a `crates/` occurrence, runs while [`is_path_char`],
/// and — if immediately followed by `::` — captures the trailing symbol token.
/// A path that does not end in `.rs` is kept only when it is a `path:line`
/// form (the trailing `:` + digits is not part of `is_path_char`, so the path
/// itself is captured up to the `.rs` or directory token). We only assert on
/// refs whose captured path ends in `.rs`, which is every Implementation cell.
fn extract_refs(text: &str) -> BTreeSet<Ref> {
    let mut out = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = text[i..].find("crates/") {
        let start = i + rel;
        // Walk the path token.
        let mut j = start;
        while j < bytes.len() {
            let c = text[j..].chars().next().unwrap();
            if is_path_char(c) {
                j += c.len_utf8();
            } else {
                break;
            }
        }
        let path = &text[start..j];
        // Only consider `.rs` file refs — that is what every Implementation
        // cell uses; a stray `crates/...` directory mention in prose is ignored.
        if path.ends_with(".rs") {
            // Optional `::symbol` suffix.
            let symbol = if text[j..].starts_with("::") {
                let sym_start = j + 2;
                let mut k = sym_start;
                while k < bytes.len() {
                    let c = text[k..].chars().next().unwrap();
                    if is_symbol_char(c) {
                        k += c.len_utf8();
                    } else {
                        break;
                    }
                }
                if k > sym_start {
                    Some(text[sym_start..k].to_string())
                } else {
                    None
                }
            } else {
                None
            };
            out.insert(Ref { path: path.to_string(), symbol });
        }
        i = j.max(start + 1);
    }
    out
}

#[test]
fn every_implementation_ref_resolves() {
    let root = repo_root();
    let doc_path = root.join("crates/deck/FEATURES.md");
    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("read FEATURES.md at {}: {e}", doc_path.display()));

    let refs = extract_refs(&doc);
    assert!(
        !refs.is_empty(),
        "extracted zero implementation refs from {} — the doc or the extractor is broken",
        doc_path.display(),
    );

    // Cache file contents so a file referenced N times is read once.
    let mut cache: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();

    let mut first_failure: Option<String> = None;
    for r in &refs {
        let contents = cache
            .entry(r.path.clone())
            .or_insert_with(|| std::fs::read_to_string(root.join(&r.path)).ok());

        let Some(contents) = contents else {
            first_failure = Some(format!(
                "unresolved ref `{}`: file does not exist at {}",
                r.path,
                root.join(&r.path).display(),
            ));
            break;
        };

        if let Some(sym) = &r.symbol
            && !contents.contains(sym.as_str())
        {
            first_failure = Some(format!(
                "unresolved ref `{}::{}`: symbol `{}` not found in {}",
                r.path,
                sym,
                sym,
                root.join(&r.path).display(),
            ));
            break;
        }
    }

    assert!(
        first_failure.is_none(),
        "FEATURES.md has a stale implementation reference (the code is truth — \
         fix the doc, not this test):\n  {}\n\nChecked {} refs total.",
        first_failure.unwrap(),
        refs.len(),
    );
}

#[cfg(test)]
mod extractor_tests {
    use super::*;

    #[test]
    fn extracts_path_and_symbol() {
        let refs = extract_refs("see `crates/deck/deck-core/src/view.rs::serve_sui_view_setup` here");
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-core/src/view.rs");
        assert_eq!(r.symbol.as_deref(), Some("serve_sui_view_setup"));
    }

    #[test]
    fn extracts_bare_file_ref() {
        let refs = extract_refs("at `crates/deck/deck-cli/src/main.rs` (no symbol)");
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-cli/src/main.rs");
        assert!(r.symbol.is_none());
    }

    #[test]
    fn ignores_non_rs_directory_mentions() {
        // A `crates/...` mention that is not a `.rs` file ref is not asserted.
        let refs = extract_refs("output lands in `crates/deck/deck-core/assets/`");
        assert!(refs.is_empty(), "directory mention must not be captured: {refs:?}");
    }

    #[test]
    fn stops_at_table_delimiters() {
        // A ref in a markdown table cell ends at the surrounding backtick/pipe.
        let refs = extract_refs(
            "| Feature | `crates/deck/deck-core/src/response.rs::DeckResponse` | desc |",
        );
        assert_eq!(refs.len(), 1);
        let r = refs.iter().next().unwrap();
        assert_eq!(r.path, "crates/deck/deck-core/src/response.rs");
        assert_eq!(r.symbol.as_deref(), Some("DeckResponse"));
    }
}
