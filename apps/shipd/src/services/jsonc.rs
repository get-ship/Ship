//! JSONC helpers for agent config parsing.
//!
//! Strips line/block comments and trailing commas so the result can be
//! fed to `serde_json`. Kept as a sibling of `agents_state` so the main
//! module stays under the 300-line cap.

/// Strip `// …` / `/* … */` comments and trailing commas from a JSONC
/// input, producing a string suitable for `serde_json::from_str`.
pub fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_block = false;
    let mut in_str = false;
    let mut it = input.chars().peekable();
    while let Some(c) = it.next() {
        if in_block {
            if c == '*' && it.peek() == Some(&'/') {
                it.next();
                in_block = false;
            }
            continue;
        }
        if in_str {
            out.push(c);
            if c == '\\' {
                if let Some(&n) = it.peek() {
                    out.push(n);
                    it.next();
                }
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            match it.peek() {
                Some(&'/') => {
                    for r in it.by_ref() {
                        if r == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some(&'*') => {
                    it.next();
                    in_block = true;
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    // Second pass: drop trailing commas before `}` or `]`.
    // Walk as bytes so we can peek ahead cheaply; preserve multi-byte UTF-8
    // sequences verbatim by copying the original byte slice (not byte-casting
    // to char, which mangles any codepoint above U+007F).
    let src = out.as_bytes();
    let mut cleaned_bytes: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if src[i] == b',' {
            let mut j = i + 1;
            while j < src.len() && src[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < src.len() && (src[j] == b'}' || src[j] == b']') {
                i += 1;
                continue;
            }
        }
        cleaned_bytes.push(src[i]);
        i += 1;
    }
    // Safe: we only dropped ASCII commas at valid char boundaries, every other
    // byte was copied unchanged from the original UTF-8 String.
    String::from_utf8(cleaned_bytes)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

pub(crate) fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub(crate) fn parse_file(
    path: &std::path::Path,
) -> Option<(String, serde_json::Value)> {
    let id = path.file_stem().and_then(|s| s.to_str())?.to_string();
    let raw = std::fs::read_to_string(path).ok()?;
    let json_str = strip_jsonc(&raw);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    Some((id, parsed))
}

pub(crate) fn extract_str_array(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| match item {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Object(map) => map
                        .get("id")
                        .or_else(|| map.get("name"))
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a collection-ish field from an agent config, tolerating the two
/// shapes Ship actually emits:
///   1. Flat: `"skills": ["a", "b"]` or `[{"id": "a"}, {"name": "b"}]`
///   2. Nested: `"skills": { "refs": [...] }` / `"mcp": { "servers": [...] }`
///
/// The `nested_keys` parameter lists candidate sub-keys to inspect when the
/// top-level value is an object; the first one that contains an array wins.
pub(crate) fn resolve_array_field(
    obj: &serde_json::Value,
    top_keys: &[&str],
    nested_keys: &[&str],
) -> Vec<String> {
    for key in top_keys {
        let Some(v) = obj.get(*key) else { continue };
        if v.is_array() {
            return extract_str_array(v);
        }
        if let Some(inner) = v.as_object() {
            for nk in nested_keys {
                if let Some(arr) = inner.get(*nk)
                    && arr.is_array()
                {
                    return extract_str_array(arr);
                }
            }
        }
    }
    Vec::new()
}
