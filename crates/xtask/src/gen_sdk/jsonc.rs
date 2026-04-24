//! Minimal JSONC parser: strip `//` line and `/* */` block comments, then
//! hand off to serde_json. Respects strings so `"http://…"` URLs survive.

use anyhow::{anyhow, Result};
use serde_json::Value;

pub fn parse(src: &str) -> Result<Value> {
    let stripped = strip_comments(src);
    let stripped = strip_trailing_commas(&stripped);
    serde_json::from_str(&stripped).map_err(|e| anyhow!("json parse: {e}"))
}

fn strip_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'/' {
                // line comment
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if next == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn strip_trailing_commas(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            continue;
        }
        if b == b',' {
            // peek forward past whitespace for `}` or `]`.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                continue; // drop the trailing comma
            }
        }
        out.push(b as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_line_and_block_comments() {
        let src = r#"{
            // leading
            "a": 1, /* inline */ "b": "http://x", /* trailing */
        }"#;
        let v = parse(src).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], "http://x");
    }

    #[test]
    fn preserves_strings_with_slashes() {
        let v = parse(r#"{ "url": "a//b/*c*/" }"#).unwrap();
        assert_eq!(v["url"], "a//b/*c*/");
    }

    #[test]
    fn trailing_comma_tolerated() {
        let v = parse(r#"{ "xs": [1, 2, 3,], "y": 4, }"#).unwrap();
        assert_eq!(v["xs"][2], 3);
        assert_eq!(v["y"], 4);
    }
}
