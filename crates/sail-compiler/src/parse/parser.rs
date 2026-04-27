//! Parser. Walks the source and emits an AST.
//!
//! See `parse::parse` (the public entry) and the AST in `ast.rs`.
//!
//! Implementation phasing per `02-compiler-architecture.md`:
//! 1. Three-block extraction (script, template, style) — implemented here.
//!    The other phases (full template parsing, script keyword scan,
//!    style attribute parsing) build on top of these blocks.

use anyhow::{Context, Result, anyhow};

use super::ast::{ScriptBlock, ScriptLang, SourceMap, StyleBlock, Template};

/// A single `.set` file's three blocks, lifted from the source by
/// position (no semantic parsing of contents yet).
#[derive(Debug, Default)]
pub struct RawBlocks {
    pub script: Option<RawBlock>,
    pub template: Option<RawBlock>,
    pub style: Option<RawBlock>,
}

/// One `<TAG ...>...</TAG>` extracted region.
#[derive(Debug, Clone)]
pub struct RawBlock {
    /// Block opening tag attributes, e.g. `lang="ts"`, `src="./x.css"`,
    /// `global`. Stored verbatim minus the wrapping spaces; consumers
    /// call helpers in this module to parse them.
    pub attrs_raw: String,
    /// The block's body — verbatim source between the open and close
    /// tags.
    pub body: String,
    /// Byte offset of the block's body start, for diagnostics that
    /// need to map body-relative spans back to file positions.
    pub body_byte_offset: usize,
}

/// Pull the three blocks out of a `.set` source string.
///
/// **A `.set` file with no `<template>` is invalid** per the spec
/// — every component must have at least a template. `<script>` and
/// `<style>` are optional. Order is convention (script → template →
/// style) but not enforced.
///
/// Returns the three optional blocks plus any leftover non-block
/// content (which is invalid and should produce a diagnostic).
pub fn extract_blocks(source: &str) -> Result<RawBlocks> {
    let mut blocks = RawBlocks::default();

    // Parse top-level blocks. We require well-formed `<TAG ...>...</TAG>`
    // with matching tags. Whitespace between blocks is permitted.
    let mut i = 0usize;
    let bytes = source.as_bytes();

    while i < bytes.len() {
        // Skip whitespace + line comments at top level (none in v1; could
        // accept Jinja-style `{# ... #}` for parity with author intent).
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] != b'<' {
            return Err(anyhow!(
                "unexpected content outside a block at byte {}: {:?}",
                i,
                &source[i..(i + 30).min(source.len())]
            ));
        }

        // Read tag name
        let tag_start = i + 1;
        let mut j = tag_start;
        while j < bytes.len() && (bytes[j].is_ascii_alphabetic() || bytes[j] == b'!') {
            j += 1;
        }
        let tag = &source[tag_start..j];

        match tag {
            "script" | "template" | "style" => {
                let block = extract_one(source, i, tag)?;
                let target = match tag {
                    "script" => &mut blocks.script,
                    "template" => &mut blocks.template,
                    "style" => &mut blocks.style,
                    _ => unreachable!(),
                };
                if target.is_some() {
                    return Err(anyhow!("duplicate <{tag}> block"));
                }
                let body_end = block.body_byte_offset + block.body.len();
                let close_end = source[body_end..]
                    .find(&format!("</{tag}>"))
                    .map(|p| body_end + p + 3 + tag.len())
                    .ok_or_else(|| anyhow!("unterminated <{tag}>"))?;
                *target = Some(block);
                i = close_end;
            }
            _ => {
                return Err(anyhow!(
                    "unknown top-level tag <{tag}>; expected <script>, <template>, or <style>"
                ));
            }
        }
    }

    Ok(blocks)
}

/// Extract one block's open-tag attrs + body. Caller must position
/// `start` at the byte index of the `<` opening the tag, and pass
/// the bare tag name (`script` | `template` | `style`).
fn extract_one(source: &str, start: usize, tag: &str) -> Result<RawBlock> {
    let bytes = source.as_bytes();
    debug_assert_eq!(bytes[start], b'<');

    // Find end of opening tag (the `>` that closes `<tag attrs...>`).
    // Mind quoted attribute values so a `>` inside an attribute doesn't
    // confuse us.
    let mut i = start + 1 + tag.len();
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if b == b'"' || b == b'\'' {
            in_str = Some(b);
            i += 1;
            continue;
        }
        if b == b'>' {
            break;
        }
        i += 1;
    }
    if i >= bytes.len() {
        return Err(anyhow!("unterminated <{tag}> opening tag"));
    }
    let open_close = i; // position of `>`
    let attrs_raw = source[(start + 1 + tag.len())..open_close]
        .trim()
        .to_string();
    let body_start = open_close + 1;

    // Find matching `</tag>`. v1 doesn't support nested tags of the
    // same name (no nested `<script>` etc.), so a simple substring
    // search is sufficient.
    let close_marker = format!("</{tag}>");
    let body_end = source[body_start..]
        .find(&close_marker)
        .map(|p| body_start + p)
        .ok_or_else(|| anyhow!("missing </{tag}> close tag"))?;

    let body = source[body_start..body_end].to_string();

    Ok(RawBlock {
        attrs_raw,
        body,
        body_byte_offset: body_start,
    })
}

/// Parse `lang="ts"` (or absent → ScriptLang::Js per the spec) from a
/// raw script block's attrs.
pub fn parse_script_lang(attrs: &str) -> ScriptLang {
    if attrs.contains("lang=\"ts\"") || attrs.contains("lang='ts'") {
        ScriptLang::Ts
    } else {
        ScriptLang::Js
    }
}

/// Parse `<style global>` and `<style src="…">`.
pub fn parse_style_attrs(attrs: &str) -> (bool, Option<String>) {
    let global = attrs.split_ascii_whitespace().any(|t| t == "global");
    let src = parse_attr_value(attrs, "src");
    (global, src)
}

fn parse_attr_value(attrs: &str, name: &str) -> Option<String> {
    // Find `name="VALUE"` or `name='VALUE'`.
    let needle_eq = format!("{name}=");
    let pos = attrs.find(&needle_eq)?;
    let after = &attrs[pos + needle_eq.len()..];
    let q = after.chars().next()?;
    if q != '"' && q != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(q)?;
    Some(rest[..end].to_string())
}

/// Lift the raw blocks into typed AST shells. Body parsing of script
/// (keyword scan), template (HTML + interpolations), and style
/// (CSS + tokens) lives in subsequent phases — this just gives us
/// the three-block scaffolding with attrs decoded.
pub fn build_shell(source: &str, raw: RawBlocks) -> super::ast::SailFile {
    super::ast::SailFile {
        script: raw.script.map(|b| ScriptBlock {
            lang: parse_script_lang(&b.attrs_raw),
            raw: b.body,
            props: Vec::new(),
            state: Vec::new(),
            derived: Vec::new(),
            data: Vec::new(),
            effects: Vec::new(),
            events: Vec::new(),
            mount: None,
            unmount: None,
            functions: Vec::new(),
            form_associated: false,
            form_lifecycle: Vec::new(),
        }),
        template: raw.template.map(|b| {
            // Template body is parsed in a later phase; for now, hold
            // it raw inside a placeholder root.
            Template {
                root: vec![super::ast::Node::Text(b.body)],
                extends: None,
            }
        }),
        style: raw.style.map(|b| {
            let (global, src) = parse_style_attrs(&b.attrs_raw);
            StyleBlock {
                raw: b.body,
                global,
                src,
            }
        }),
        source_map: SourceMap {
            source: source.to_string(),
        },
    }
}

/// Public entry point for the parser. Combines block extraction +
/// shell building. Body parsing of each block lands incrementally.
pub fn parse_file(source: &str) -> Result<super::ast::SailFile> {
    let raw = extract_blocks(source).context("extract blocks")?;
    let mut file = build_shell(source, raw);
    if let Some(script) = file.script.as_mut() {
        super::script::scan(script);
    }
    if let Some(template) = file.template.as_mut() {
        // The shell stuffed the raw body into a single Text node — pull
        // it back out and run the proper template parser.
        if let Some(super::ast::Node::Text(body)) = template.root.first().cloned() {
            *template = super::template::parse(&body)?;
        }
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_three_blocks_in_canonical_order() {
        let src = r#"<script lang="ts">
const x = 1
</script>

<template>
<div>hello</div>
</template>

<style>
.foo { color: red; }
</style>
"#;
        let raw = extract_blocks(src).unwrap();
        assert!(raw.script.is_some());
        assert!(raw.template.is_some());
        assert!(raw.style.is_some());
        assert!(raw.script.as_ref().unwrap().body.contains("const x = 1"));
        assert!(raw.template.as_ref().unwrap().body.contains("hello"));
        assert!(raw.style.as_ref().unwrap().body.contains("color: red"));
    }

    #[test]
    fn template_only_is_valid_blocks() {
        let src = r#"<template><span>hi</span></template>"#;
        let raw = extract_blocks(src).unwrap();
        assert!(raw.script.is_none());
        assert!(raw.template.is_some());
        assert!(raw.style.is_none());
    }

    #[test]
    fn empty_script_block_extracts_empty_body() {
        let src = "<script></script><template>x</template>";
        let raw = extract_blocks(src).unwrap();
        assert_eq!(raw.script.unwrap().body, "");
    }

    #[test]
    fn duplicate_block_errors() {
        let src = "<template>a</template><template>b</template>";
        let err = extract_blocks(src).unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn unknown_top_level_tag_errors() {
        let src = "<weird>nope</weird>";
        let err = extract_blocks(src).unwrap_err();
        assert!(err.to_string().contains("unknown top-level tag"));
    }

    #[test]
    fn unterminated_block_errors() {
        let src = "<template>oh no";
        let err = extract_blocks(src).unwrap_err();
        assert!(err.to_string().contains("missing </template>"));
    }

    #[test]
    fn order_independent() {
        // Style → template → script — daemon doesn't enforce order.
        let src = "<style>a{}</style><template>b</template><script>c</script>";
        let raw = extract_blocks(src).unwrap();
        assert!(raw.script.is_some());
        assert!(raw.template.is_some());
        assert!(raw.style.is_some());
    }

    #[test]
    fn parses_script_lang_ts() {
        assert_eq!(parse_script_lang(r#"lang="ts""#), ScriptLang::Ts);
        assert_eq!(parse_script_lang("lang='ts'"), ScriptLang::Ts);
        assert_eq!(parse_script_lang(""), ScriptLang::Js);
        assert_eq!(parse_script_lang(r#"lang="js""#), ScriptLang::Js);
    }

    #[test]
    fn parses_style_global() {
        let (global, src) = parse_style_attrs("global");
        assert!(global);
        assert!(src.is_none());
    }

    #[test]
    fn parses_style_src() {
        let (global, src) = parse_style_attrs(r#"src="./shared.css""#);
        assert!(!global);
        assert_eq!(src.as_deref(), Some("./shared.css"));
    }

    #[test]
    fn parses_style_global_and_src_together() {
        let (global, src) = parse_style_attrs(r#"global src="./shared.css""#);
        assert!(global);
        assert_eq!(src.as_deref(), Some("./shared.css"));
    }

    #[test]
    fn quoted_attribute_with_gt_does_not_confuse_block_close() {
        // The `>` inside the attribute value would break a naive parser
        // that scans for `>` to close the opening tag.
        let src = r#"<script lang="ts" data-x="<">const x=1</script>"#;
        let raw = extract_blocks(src).unwrap();
        let s = raw.script.unwrap();
        assert!(s.attrs_raw.contains("data-x=\"<\""));
        assert_eq!(s.body, "const x=1");
    }

    #[test]
    fn parse_file_builds_typed_shell() {
        let src = r#"<script lang="ts">const x = 1</script>
<template><div>hi</div></template>
<style global>.x{}</style>"#;
        let file = parse_file(src).unwrap();
        let s = file.script.unwrap();
        assert!(matches!(s.lang, ScriptLang::Ts));
        assert!(s.raw.contains("const x = 1"));
        let style = file.style.unwrap();
        assert!(style.global);
        assert!(style.src.is_none());
    }
}
