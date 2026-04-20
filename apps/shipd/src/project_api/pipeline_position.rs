//! Resolve JSON Pointers against a JSONC source to (line, column).
//!
//! Used to place schema-validation markers in the editor. The resolver
//! operates on a position-preserving stripped copy of the source so
//! line/column in the cleaned buffer maps 1:1 to the original JSONC.

/// Map a JSON Pointer (e.g. `/steps/0/stepz`) to 1-based (line, column)
/// in the original JSONC source. Returns None when the pointer doesn't
/// resolve to a valid location — schema errors then fall back to no
/// positional marker.
pub fn resolve_pointer(jsonc_source: &str, pointer: &str) -> Option<(u32, u32)> {
    if pointer.is_empty() {
        return Some((1, 1));
    }
    let cleaned = compiler::jsonc::strip_jsonc_comments_preserve_positions(jsonc_source);
    let bytes = cleaned.as_bytes();
    let segments = decode_pointer(pointer);

    let mut pos = skip_ws(bytes, 0);
    for seg in &segments {
        pos = match bytes.get(pos)? {
            b'{' => find_object_value(bytes, pos, seg)?,
            b'[' => {
                let idx: usize = seg.parse().ok()?;
                find_array_value(bytes, pos, idx)?
            }
            _ => return None,
        };
        pos = skip_ws(bytes, pos);
    }

    Some(offset_to_line_col(&cleaned, pos))
}

fn decode_pointer(pointer: &str) -> Vec<String> {
    pointer
        .trim_start_matches('/')
        .split('/')
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn find_object_value(bytes: &[u8], obj_start: usize, key: &str) -> Option<usize> {
    let mut pos = obj_start.checked_add(1)?;
    loop {
        pos = skip_ws(bytes, pos);
        match bytes.get(pos) {
            Some(b'}') | None => return None,
            Some(b'"') => {}
            _ => return None,
        }
        let (parsed_key, after_key) = parse_string(bytes, pos)?;
        pos = skip_ws(bytes, after_key);
        if bytes.get(pos) != Some(&b':') {
            return None;
        }
        pos = skip_ws(bytes, pos + 1);
        if parsed_key == key {
            return Some(pos);
        }
        pos = skip_value(bytes, pos)?;
        pos = skip_ws(bytes, pos);
        if bytes.get(pos) == Some(&b',') {
            pos += 1;
        } else {
            return None;
        }
    }
}

fn find_array_value(bytes: &[u8], arr_start: usize, index: usize) -> Option<usize> {
    let mut pos = arr_start.checked_add(1)?;
    for i in 0..=index {
        pos = skip_ws(bytes, pos);
        if bytes.get(pos) == Some(&b']') {
            return None;
        }
        if i == index {
            return Some(pos);
        }
        pos = skip_value(bytes, pos)?;
        pos = skip_ws(bytes, pos);
        if bytes.get(pos) == Some(&b',') {
            pos += 1;
        } else {
            return None;
        }
    }
    None
}

fn skip_value(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = start;
    match *bytes.get(pos)? {
        b'"' => {
            let (_, end) = parse_string(bytes, pos)?;
            Some(end)
        }
        open @ (b'{' | b'[') => {
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 1i32;
            pos += 1;
            while depth > 0 {
                match *bytes.get(pos)? {
                    b'"' => {
                        let (_, end) = parse_string(bytes, pos)?;
                        pos = end;
                        continue;
                    }
                    c if c == open => depth += 1,
                    c if c == close => depth -= 1,
                    _ => {}
                }
                pos += 1;
            }
            Some(pos)
        }
        _ => {
            // Number / true / false / null — read until a JSON structural char.
            while pos < bytes.len() {
                let c = bytes[pos];
                if c == b',' || c == b'}' || c == b']' || c.is_ascii_whitespace() {
                    break;
                }
                pos += 1;
            }
            Some(pos)
        }
    }
}

fn parse_string(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    if bytes.get(start)? != &b'"' {
        return None;
    }
    let mut pos = start + 1;
    let mut buf = Vec::new();
    while pos < bytes.len() {
        match bytes[pos] {
            b'"' => {
                let s = String::from_utf8_lossy(&buf).into_owned();
                return Some((s, pos + 1));
            }
            b'\\' => {
                pos += 1;
                match *bytes.get(pos)? {
                    b'"' => buf.push(b'"'),
                    b'\\' => buf.push(b'\\'),
                    b'/' => buf.push(b'/'),
                    b'n' => buf.push(b'\n'),
                    b't' => buf.push(b'\t'),
                    b'r' => buf.push(b'\r'),
                    b'b' => buf.push(8),
                    b'f' => buf.push(12),
                    b'u' => {
                        // Copy the 4 hex digits verbatim — key comparison doesn't
                        // need the actual codepoint; byte-for-byte is enough.
                        for _ in 0..4 {
                            pos += 1;
                            buf.push(*bytes.get(pos)?);
                        }
                    }
                    other => buf.push(other),
                }
                pos += 1;
            }
            c => {
                buf.push(c);
                pos += 1;
            }
        }
    }
    None
}

fn skip_ws(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut col = 1u32;
    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_pointer_resolves_to_start() {
        let src = r#"{ "a": 1 }"#;
        assert_eq!(resolve_pointer(src, ""), Some((1, 1)));
    }

    #[test]
    fn top_level_key() {
        let src = "{\n  \"name\": \"x\",\n  \"steps\": []\n}";
        let (line, _col) = resolve_pointer(src, "/steps").unwrap();
        assert_eq!(line, 3);
    }

    #[test]
    fn array_index() {
        let src = "{\n  \"steps\": [\n    { \"id\": \"a\" },\n    { \"id\": \"b\" }\n  ]\n}";
        let (line, _) = resolve_pointer(src, "/steps/1").unwrap();
        assert_eq!(line, 4);
    }

    #[test]
    fn nested_path() {
        let src = "{\n  \"steps\": [\n    { \"id\": \"a\" }\n  ]\n}";
        let (line, _) = resolve_pointer(src, "/steps/0/id").unwrap();
        assert_eq!(line, 3);
    }

    #[test]
    fn path_through_line_comment() {
        let src = "{\n  // this is a comment\n  \"name\": \"x\",\n  \"steps\": []\n}";
        let (line, _) = resolve_pointer(src, "/steps").unwrap();
        assert_eq!(line, 4);
    }

    #[test]
    fn missing_key_returns_none() {
        let src = r#"{ "name": "x" }"#;
        assert_eq!(resolve_pointer(src, "/missing"), None);
    }

    #[test]
    fn array_out_of_bounds_returns_none() {
        let src = r#"{ "steps": [1, 2] }"#;
        assert_eq!(resolve_pointer(src, "/steps/5"), None);
    }

    #[test]
    fn pointer_with_escaped_slash() {
        let src = "{\n  \"a/b\": 42\n}";
        let (line, _) = resolve_pointer(src, "/a~1b").unwrap();
        assert_eq!(line, 2);
    }
}
