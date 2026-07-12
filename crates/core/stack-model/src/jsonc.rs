//! JSONC (JSON with Comments) support.
//!
//! Strips `//` line comments and `/* */` block comments plus trailing commas
//! from a JSONC string, then deserialises with `serde_json`.

/// Strip JSONC comments and trailing commas, returning valid JSON.
///
/// Handles:
/// - `// line comments`
/// - `/* block comments */`
/// - Trailing commas before `]` or `}`
/// - Strings (comments inside strings are preserved)
pub fn strip_jsonc_comments(input: &str) -> String {
    strip_jsonc_comments_inner(input, false)
}

/// Strip JSONC comments while preserving every source character's byte
/// offset: comments are replaced with whitespace, not removed.
///
/// Use this when you need line/column positions in the cleaned JSON to
/// match the original JSONC exactly — e.g. for editor diagnostics that
/// render on the user's raw buffer.
pub fn strip_jsonc_comments_preserve_positions(input: &str) -> String {
    strip_jsonc_comments_inner(input, true)
}

fn strip_jsonc_comments_inner(input: &str, preserve_positions: bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    // Helper — emit `c` when keeping positions, otherwise drop it (except
    // for newlines, which are always kept so line counts stay correct).
    let blank = |c: char, keep: bool, out: &mut String| {
        if c == '\n' || c == '\r' {
            out.push(c);
        } else if keep {
            out.push(' ');
        }
    };

    while let Some(&(_, ch)) = chars.peek() {
        match ch {
            // String literal — copy verbatim, handling escapes
            '"' => {
                out.push('"');
                chars.next();
                while let Some(&(_, c)) = chars.peek() {
                    if c == '\\' {
                        chars.next();
                        out.push(c);
                        if let Some((_, esc)) = chars.next() {
                            out.push(esc);
                        }
                    } else if c == '"' {
                        out.push('"');
                        chars.next();
                        break;
                    } else {
                        out.push(c);
                        chars.next();
                    }
                }
            }
            // Possible comment start
            '/' => {
                chars.next();
                match chars.peek().map(|&(_, c)| c) {
                    Some('/') => {
                        // Line comment — skip to end of line
                        if preserve_positions {
                            out.push(' ');
                            out.push(' ');
                        }
                        chars.next();
                        while let Some(&(_, c)) = chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            blank(c, preserve_positions, &mut out);
                            chars.next();
                        }
                    }
                    Some('*') => {
                        // Block comment — skip to */
                        if preserve_positions {
                            out.push(' ');
                            out.push(' ');
                        }
                        chars.next();
                        let mut prev = '\0';
                        for (_, c) in chars.by_ref() {
                            if prev == '*' && c == '/' {
                                if preserve_positions {
                                    // The `*` was emitted below as blank;
                                    // emit the closing `/` too so char count matches.
                                    out.push(' ');
                                }
                                break;
                            }
                            blank(c, preserve_positions, &mut out);
                            prev = c;
                        }
                    }
                    _ => {
                        out.push('/');
                    }
                }
            }
            _ => {
                out.push(ch);
                chars.next();
            }
        }
    }

    // Strip trailing commas before ] or } — only in the strip mode.
    // Preserving positions means keeping every byte in place, commas included.
    if preserve_positions {
        out
    } else {
        strip_trailing_commas(&out)
    }
}

/// Remove trailing commas: `,` followed by optional whitespace then `]` or `}`.
fn strip_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == ',' {
            // Look ahead past whitespace for ] or }
            let mut j = i + 1;
            while j < len
                && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r')
            {
                j += 1;
            }
            if j < len && (chars[j] == ']' || chars[j] == '}') {
                // Skip the comma, keep the whitespace
                i += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    out
}

/// Parse a JSONC string into a deserializable type.
pub fn from_jsonc_str<T: serde::de::DeserializeOwned>(s: &str) -> serde_json::Result<T> {
    let json = strip_jsonc_comments(s);
    serde_json::from_str(&json)
}

/// Detect config format from file extension.
pub fn is_jsonc_path(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("jsonc") | Some("json")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_line_comments() {
        let input = r#"{
  // This is a comment
  "key": "value" // inline comment
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["key"], "value");
    }

    #[test]
    fn strip_block_comments() {
        let input = r#"{
  /* block comment */
  "key": /* inline */ "value"
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["key"], "value");
    }

    #[test]
    fn strip_trailing_comma() {
        let input = r#"{
  "a": 1,
  "b": 2,
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 2);
    }

    #[test]
    fn strip_trailing_comma_in_array() {
        let input = r#"{ "arr": [1, 2, 3,] }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["arr"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn preserve_strings_with_slashes() {
        let input = r#"{ "url": "https://example.com/path" }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["url"], "https://example.com/path");
    }

    #[test]
    fn preserve_escaped_quotes_in_strings() {
        let input = r#"{ "msg": "say \"hello\"" }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["msg"], r#"say "hello""#);
    }

    #[test]
    fn preserve_positions_keeps_byte_count() {
        let input = r#"{
  // a comment
  "key": "value" // end
}"#;
        let cleaned = strip_jsonc_comments_preserve_positions(input);
        assert_eq!(
            cleaned.len(),
            input.len(),
            "preserve_positions must not shrink the source"
        );
        // Newlines still at same byte offsets.
        let nl_in: Vec<_> = input.match_indices('\n').map(|(i, _)| i).collect();
        let nl_out: Vec<_> = cleaned.match_indices('\n').map(|(i, _)| i).collect();
        assert_eq!(nl_in, nl_out);
        // The cleaned text is still JSONC (trailing commas preserved).
        // strip_jsonc_comments on the cleaned buffer produces valid JSON.
        let final_json = strip_jsonc_comments(&cleaned);
        let v: serde_json::Value = serde_json::from_str(&final_json).unwrap();
        assert_eq!(v["key"], "value");
    }

    #[test]
    fn from_jsonc_str_works() {
        let input = r#"{
  // Ship manifest
  "module": {
    "name": "github.com/owner/repo",
    "version": "1.0.0",
  },
}"#;
        let v: serde_json::Value = from_jsonc_str(input).unwrap();
        assert_eq!(v["module"]["name"], "github.com/owner/repo");
    }
}

use std::path::Path;

use anyhow::{Result, anyhow};

// ── JSON merge helpers (single canonical impl; emission + resolve both use) ──

/// Recursively merge `patch` into `base` (patch wins on scalar conflict).
pub fn merge_json(base: &mut serde_json::Value, patch: &serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                merge_json(b.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

/// Read an existing JSON file (or start with `{}`), merge `patch` in, write back.
pub fn merge_json_file(path: &Path, patch: &serde_json::Value) -> Result<()> {
    let mut existing: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?)
            .map_err(|e| anyhow!("failed to parse {}: {}", path.display(), e))?
    } else {
        serde_json::json!({})
    };
    merge_json(&mut existing, patch);
    std::fs::write(path, serde_json::to_string_pretty(&existing)?)?;
    Ok(())
}
