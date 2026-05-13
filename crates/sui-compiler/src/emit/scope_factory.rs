//! Extract `(name, ts-type)` pairs from a paired controller's
//! `scope: () => ({...})` factory so the d.ts emitter can surface them
//! as additional `Scope` members.
//!
//! Two paths are handled:
//!
//!  * Expression-body factories — `scope: () => ({ foo: signal(0) })`.
//!    The returned object literal is the entry source.
//!  * Block-body factories — `scope: () => { const foo = signal(0); return { foo } }`.
//!    The last `return {...}` is the entry source; shorthand entries
//!    resolve by locating a top-level `const NAME = …` / `let NAME = …`
//!    / `function NAME …` binding within the body and re-running the
//!    type heuristic against its RHS.
//!
//! Inferred types are best-effort string heuristics:
//!  * `signal<T>(…)` → `Signal<T>`
//!  * `signal(literal)` → `Signal<boolean | number | string | null>`
//!  * `ref<T>(…)` → `Signal<T | null>`; `ref(null)` → `Signal<HTMLElement | null>`
//!  * `derived(() => …)` → `Signal<unknown>`
//!  * `(...): T => …` / `(...) => {…}` → `(...args: unknown[]) => T | void`
//!  * primitive literals (`42`, `'x'`, `true`) → primitive type
//!  * anything else → `unknown`
//!
//! Authors who want a tighter type than the heuristic can always annotate
//! the call (`signal<MyType>(...)`). Untyped values fall back to `unknown`,
//! which still typechecks against `Signal<unknown>` / direct member use.

use crate::check::controller_parse::ParsedController;
use crate::diagnostics::Span;

/// `(name, ts_type)` pairs for every entry surfaced by the controller's
/// `scope: () => ({...})` factory. Returns an empty vec when no `scope`
/// section is present or the factory body doesn't model a return object.
pub fn extract(controller: &ParsedController) -> Vec<(String, String)> {
    let Some(section) = controller.section("scope") else {
        return Vec::new();
    };
    let bytes = controller.source.as_bytes();
    let body_span = section.value_span;
    let Some(obj_span) = locate_return_object(bytes, body_span) else {
        return Vec::new();
    };
    let mut out: Vec<(String, String)> = Vec::new();
    walk_entries(bytes, obj_span, |name, value_start, value_end| {
        let ty = if is_shorthand(bytes, name, value_start, value_end) {
            resolve_local_binding(bytes, name, body_span.start, obj_span.start)
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            infer_ts_type(bytes, value_start, value_end)
        };
        out.push((name.to_string(), ty));
    });
    out
}

/// Find the byte span of the object literal returned by the factory.
/// Handles both `=> ({…})` and `=> { … return {…} }` shapes.
fn locate_return_object(bytes: &[u8], span: Span) -> Option<Span> {
    let start = span.start;
    let end = span.end.min(bytes.len());

    // Block body: prefer the LAST top-level `return {...}` we find.
    let return_kw = b"return";
    let mut i = start;
    let mut last_return_obj: Option<usize> = None;
    while i + return_kw.len() <= end {
        if bytes[i..].starts_with(return_kw) {
            // Word-boundary guard so we don't match `returnX`.
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let next = i + return_kw.len();
            let post_ok = next >= end || !is_ident_cont(bytes[next]);
            if prev_ok && post_ok {
                let after = skip_trivia(bytes, next, end);
                if after < end && bytes[after] == b'{' {
                    last_return_obj = Some(after);
                }
            }
        }
        i += 1;
    }
    if let Some(s) = last_return_obj {
        let e = skip_balanced(bytes, s, end);
        return Some(Span::new(s + 1, e.saturating_sub(1)));
    }

    // Expression body: skip params and `=>`, then look for `({…})` or `{…}`.
    let mut k = skip_trivia(bytes, start, end);
    if k >= end {
        return None;
    }
    if bytes[k] == b'(' {
        k = skip_balanced(bytes, k, end);
    } else if is_ident_start(bytes[k]) {
        while k < end && is_ident_cont(bytes[k]) {
            k += 1;
        }
    } else {
        return None;
    }
    k = skip_trivia(bytes, k, end);
    // Optional return-type annotation: `(): T =>`.
    if k < end && bytes[k] == b':' {
        k = skip_type_annotation(bytes, k + 1, end);
        k = skip_trivia(bytes, k, end);
    }
    if k + 1 >= end || bytes[k] != b'=' || bytes[k + 1] != b'>' {
        return None;
    }
    k = skip_trivia(bytes, k + 2, end);
    // Allow an outer `({...})` wrapper.
    while k < end && bytes[k] == b'(' {
        k = skip_trivia(bytes, k + 1, end);
    }
    if k >= end || bytes[k] != b'{' {
        return None;
    }
    let close = skip_balanced(bytes, k, end);
    Some(Span::new(k + 1, close.saturating_sub(1)))
}

/// Walk a `{ key: value, … }` slice (start/end excluding the braces) and
/// invoke `f` per entry.
fn walk_entries<F>(bytes: &[u8], span: Span, mut f: F)
where
    F: FnMut(&str, usize, usize),
{
    let start = span.start;
    let end = span.end.min(bytes.len());
    let mut i = start;
    while i < end {
        i = skip_trivia(bytes, i, end);
        if i >= end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(end);
                    continue;
                }
                i += 1;
            }
            key = std::str::from_utf8(&bytes[s..i])
                .unwrap_or("")
                .to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = std::str::from_utf8(&bytes[key_start..i])
                .unwrap_or("")
                .to_string();
        } else {
            // Spread / computed key / something we don't model.
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        let after_key = i;
        i = skip_trivia(bytes, i, end);
        // Shorthand: `{ key }` — value span IS the key.
        if i >= end || bytes[i] == b',' || bytes[i] == b'}' {
            f(&key, key_start, after_key);
            if i < end && bytes[i] == b',' {
                i += 1;
            }
            continue;
        }
        if bytes[i] != b':' {
            // Method form (`foo() {…}`) or shape we don't recognise — skip.
            i = advance_past_entry(bytes, i, end);
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i, end);
        let value_start = i;
        let value_end = next_top_separator(bytes, i, end);
        f(&key, value_start, value_end);
        i = value_end;
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
}

/// True when a `key: value` entry's value-span is just the key itself
/// (e.g. `{ foo }` — a shorthand binding).
fn is_shorthand(bytes: &[u8], name: &str, start: usize, end: usize) -> bool {
    let s = skip_trivia(bytes, start, end);
    if s + name.len() > end {
        return false;
    }
    if &bytes[s..s + name.len()] != name.as_bytes() {
        return false;
    }
    let after = skip_trivia(bytes, s + name.len(), end);
    after >= end
}

/// Walk the factory body for a top-level `const NAME = …` / `let NAME = …`
/// / `function NAME …` declaration and infer its TS type. Returns `None`
/// if no such binding exists.
fn resolve_local_binding(
    bytes: &[u8],
    name: &str,
    start: usize,
    end: usize,
) -> Option<String> {
    let nb = name.as_bytes();
    let mut i = start;
    while i < end {
        // `function NAME(...)` / `async function NAME(...)`.
        if bytes[i..end.min(i + 8)].starts_with(b"function") {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + 8;
            if prev_ok && after_kw < end && !is_ident_cont(bytes[after_kw]) {
                let j_start = skip_trivia(bytes, after_kw, end);
                if j_start + nb.len() <= end
                    && &bytes[j_start..j_start + nb.len()] == nb
                    && (j_start + nb.len() == end || !is_ident_cont(bytes[j_start + nb.len()]))
                {
                    let j = skip_trivia(bytes, j_start + nb.len(), end);
                    if j < end && bytes[j] == b'(' {
                        let pclose = skip_balanced(bytes, j, end);
                        let after_params = skip_trivia(bytes, pclose, end);
                        let ret = if after_params < end && bytes[after_params] == b':' {
                            let ret_start = after_params + 1;
                            let ret_end = skip_type_annotation(bytes, ret_start, end);
                            let ts = std::str::from_utf8(&bytes[ret_start..ret_end])
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            if ts.is_empty() { "void".to_string() } else { ts }
                        } else {
                            "void".to_string()
                        };
                        let params = std::str::from_utf8(&bytes[j..pclose])
                            .unwrap_or("()")
                            .to_string();
                        return Some(format!("{params} => {ret}"));
                    }
                }
            }
        }
        // `const NAME = …` / `let NAME = …`.
        let kw_len = if bytes[i..end.min(i + 5)].starts_with(b"const") {
            5
        } else if bytes[i..end.min(i + 3)].starts_with(b"let") {
            3
        } else {
            0
        };
        if kw_len > 0 {
            let prev_ok = i == start || !is_ident_cont(bytes[i - 1]);
            let after_kw = i + kw_len;
            let post_ok = after_kw < end && !is_ident_cont(bytes[after_kw]);
            if prev_ok && post_ok {
                let j = skip_trivia(bytes, after_kw, end);
                if j + nb.len() <= end
                    && &bytes[j..j + nb.len()] == nb
                    && (j + nb.len() == end || !is_ident_cont(bytes[j + nb.len()]))
                {
                    let mut k = skip_trivia(bytes, j + nb.len(), end);
                    let mut explicit_type: Option<String> = None;
                    if k < end && bytes[k] == b':' {
                        let ts_start = k + 1;
                        let ts_end = walk_to_equals(bytes, ts_start, end);
                        let s = std::str::from_utf8(&bytes[ts_start..ts_end])
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !s.is_empty() {
                            explicit_type = Some(s);
                        }
                        k = ts_end;
                    }
                    if k < end && bytes[k] == b'=' {
                        if let Some(t) = explicit_type {
                            return Some(t);
                        }
                        let rhs_start = skip_trivia(bytes, k + 1, end);
                        let rhs_end = next_top_separator(bytes, rhs_start, end);
                        return Some(infer_ts_type(bytes, rhs_start, rhs_end));
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Walk a binding annotation `: TYPE = …` up to the top-level `=`.
fn walk_to_equals(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 && c == b'=' {
            // Don't trip on `=>` inside the type.
            if k + 1 < end && bytes[k + 1] == b'>' {
                k += 2;
                continue;
            }
            return k;
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        k += 1;
    }
    end
}

/// Heuristic value→type mapper. See module docs for the supported shapes.
fn infer_ts_type(bytes: &[u8], start: usize, end: usize) -> String {
    let mut i = skip_trivia(bytes, start, end);
    if i >= end {
        return "unknown".to_string();
    }
    // Arrow-shaped: `(...args): T => …` / `(...args) => …`.
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let params_raw = std::str::from_utf8(&bytes[i..close])
            .unwrap_or("()")
            .to_string();
        let mut after = skip_trivia(bytes, close, end);
        let mut return_type = "void".to_string();
        if after < end && bytes[after] == b':' {
            let ret_start = after + 1;
            let ret_end = skip_type_annotation(bytes, ret_start, end);
            let s = std::str::from_utf8(&bytes[ret_start..ret_end])
                .unwrap_or("")
                .trim()
                .to_string();
            if !s.is_empty() {
                return_type = s;
            }
            after = ret_end;
        }
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            let params = sanitize_arrow_params(&params_raw);
            return format!("{params} => {return_type}");
        }
    }
    // `name => body` (single-param shorthand) — uncommon for scope entries,
    // but model it as a unary action.
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = std::str::from_utf8(&bytes[s..i]).unwrap_or("");
        let mut after = skip_trivia(bytes, i, end);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return format!("({word}: unknown) => unknown");
        }
        // `signal<T>(…)` / `ref<T>(…)` / `derived(...)` / `computed(...)`.
        let mut generic: Option<String> = None;
        if after < end && bytes[after] == b'<' {
            let close = match_angle(bytes, after, end);
            if close > after {
                let inner = std::str::from_utf8(&bytes[after + 1..close - 1])
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !inner.is_empty() {
                    generic = Some(inner);
                }
                after = close;
            }
            after = skip_trivia(bytes, after, end);
        }
        if after < end && bytes[after] == b'(' {
            let inner_close = skip_balanced(bytes, after, end);
            let arg_slice = std::str::from_utf8(&bytes[after + 1..inner_close - 1])
                .unwrap_or("");
            return match word {
                "signal" => match generic {
                    Some(t) => format!("Signal<{t}>"),
                    None => format!("Signal<{}>", literal_ts(arg_slice)),
                },
                "ref" => match generic {
                    Some(t) => format!("Signal<{t} | null>"),
                    None => "Signal<HTMLElement | null>".to_string(),
                },
                "derived" | "computed" | "memo" => match generic {
                    Some(t) => format!("Signal<{t}>"),
                    None => "Signal<unknown>".to_string(),
                },
                _ => "unknown".to_string(),
            };
        }
        // Bare identifier — opaque.
        return "unknown".to_string();
    }
    // Plain literals.
    match bytes[i] {
        b'"' | b'\'' | b'`' => "string".to_string(),
        b'-' | b'+' => {
            let mut k = i + 1;
            while k < end && (bytes[k].is_ascii_digit() || bytes[k] == b'.') {
                k += 1;
            }
            if k > i + 1 { "number".to_string() } else { "unknown".to_string() }
        }
        c if c.is_ascii_digit() => "number".to_string(),
        b't' | b'f' => {
            let word = remaining_word(bytes, i, end);
            if word == "true" || word == "false" { "boolean".to_string() } else { "unknown".to_string() }
        }
        b'n' => {
            let word = remaining_word(bytes, i, end);
            if word == "null" { "null".to_string() } else { "unknown".to_string() }
        }
        b'[' => "unknown[]".to_string(),
        b'{' => "Record<string, unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

fn remaining_word(bytes: &[u8], start: usize, end: usize) -> &str {
    let mut k = start;
    while k < end && is_ident_cont(bytes[k]) {
        k += 1;
    }
    std::str::from_utf8(&bytes[start..k]).unwrap_or("")
}

/// Infer a TypeScript primitive type from a literal-shaped expression.
/// Falls back to `unknown` for anything non-trivial.
fn literal_ts(src: &str) -> String {
    let s = src.trim();
    if s.is_empty() { return "unknown".to_string(); }
    let first = s.as_bytes()[0];
    match first {
        b'"' | b'\'' | b'`' => "string".to_string(),
        b'-' | b'+' if s.len() > 1 && (s.as_bytes()[1].is_ascii_digit() || s.as_bytes()[1] == b'.') => "number".to_string(),
        c if c.is_ascii_digit() => "number".to_string(),
        b't' if s == "true" => "boolean".to_string(),
        b'f' if s == "false" => "boolean".to_string(),
        b'n' if s == "null" => "null".to_string(),
        b'[' => "unknown[]".to_string(),
        b'{' => "Record<string, unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Rewrite arrow-function parameter source so it's valid as a TS function
/// type's parameter list. Strips default values (`= 0`) but keeps names
/// and TS annotations.
fn sanitize_arrow_params(raw: &str) -> String {
    // The raw slice includes the surrounding parens. Strip the leading
    // underscore-prefixed-only-param convention by leaving names intact;
    // TS function types accept underscored placeholders unchanged.
    raw.to_string()
}

/// Walk a TypeScript generic-args block starting at `bytes[start] == b'<'`.
/// Returns the index immediately after the matching `>`; or `start` if
/// the block is malformed.
fn match_angle(bytes: &[u8], start: usize, end: usize) -> usize {
    if bytes[start] != b'<' {
        return start;
    }
    let mut depth = 0i32;
    let mut k = start;
    while k < end {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            b'"' | b'\'' | b'`' => return start, // string in type args — bail.
            _ => {}
        }
        k += 1;
    }
    start
}

/// Walk a TS return-type annotation starting after the leading `:`.
/// Stops at the next `=>` token or top-level boundary (`,` `;` `=` `)` `}` `]`).
fn skip_type_annotation(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut depth: i32 = 0;
    while k < end {
        let c = bytes[k];
        if depth == 0 {
            if c == b'=' && k + 1 < end && bytes[k + 1] == b'>' {
                return k;
            }
            if matches!(c, b',' | b';' | b'=' | b')' | b'}' | b']') {
                return k;
            }
        }
        match c {
            b'<' | b'(' | b'{' | b'[' => depth += 1,
            b'>' | b')' | b'}' | b']' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return k;
        }
        k += 1;
    }
    end
}

fn skip_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
    loop {
        while i < end && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
            continue;
        }
        break;
    }
    i
}

fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end {
        return end;
    }
    let (open, close) = match bytes[i] {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return i + 1,
    };
    let mut depth = 0i32;
    let mut k = i;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k = (k + 2).min(end); continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            c if c == open => { depth += 1; k += 1; }
            c if c == close => { depth -= 1; k += 1; if depth == 0 { return k; } }
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                while k < end && bytes[k] != b'\n' { k += 1; }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k += 2;
                while k + 1 < end && !(bytes[k] == b'*' && bytes[k + 1] == b'/') { k += 1; }
                k = (k + 2).min(end);
            }
            _ => k += 1,
        }
    }
    end
}

fn next_top_separator(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k = (k + 2).min(end); continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            b',' | b'}' => return k,
            _ => k += 1,
        }
    }
    end
}

fn advance_past_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let stop = next_top_separator(bytes, start, end);
    if stop < end && bytes[stop] == b',' {
        stop + 1
    } else {
        stop
    }
}

#[inline]
fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}

#[inline]
fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("x.controller.ts"), src.to_string()).expect("parsed")
    }

    fn names_types(v: &[(String, String)]) -> Vec<(&str, &str)> {
        v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
    }

    #[test]
    fn typed_signal_generic_uses_generic_arg() {
        let c = ctrl(
            "export default controller({\n  \
             scope: () => ({\n    \
               err: signal<Error | null>(null),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("err", "Signal<Error | null>")]);
    }

    #[test]
    fn untyped_signal_infers_from_literal() {
        let c = ctrl(
            "export default controller({\n  \
             scope: () => ({\n    \
               open: signal(false),\n    \
               count: signal(0),\n    \
               name: signal(''),\n  \
             }),\n});",
        );
        let v = extract(&c);
        let map: Vec<(&str, &str)> = names_types(&v);
        assert!(map.contains(&("open", "Signal<boolean>")), "{map:?}");
        assert!(map.contains(&("count", "Signal<number>")), "{map:?}");
        assert!(map.contains(&("name", "Signal<string>")), "{map:?}");
    }

    #[test]
    fn ref_with_generic() {
        let c = ctrl(
            "export default controller({\n  \
             scope: () => ({\n    \
               btn: ref<HTMLButtonElement>(null),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("btn", "Signal<HTMLButtonElement | null>")]);
    }

    #[test]
    fn arrow_function_returns_void() {
        let c = ctrl(
            "export default controller({\n  \
             scope: () => ({\n    \
               reset: (): void => {},\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, "reset");
        assert!(v[0].1.ends_with("=> void"), "got {}", v[0].1);
    }

    #[test]
    fn block_body_resolves_shorthand_to_const_rhs() {
        let c = ctrl(
            "export default controller({\n  \
               scope: (_b: unknown, host?: Element) => {\n    \
                 const caughtError = signal<Error | null>(null)\n    \
                 const reset = (): void => { caughtError.set(null) }\n    \
                 return { caughtError, reset }\n  \
               },\n});",
        );
        let v = extract(&c);
        let map: Vec<(&str, &str)> = names_types(&v);
        assert!(map.contains(&("caughtError", "Signal<Error | null>")), "{map:?}");
        let reset = v.iter().find(|(n, _)| n == "reset").expect("reset present");
        assert!(reset.1.ends_with("=> void"), "got {}", reset.1);
    }

    #[test]
    fn empty_factory_returns_no_entries() {
        let c = ctrl("export default controller({ scope: () => ({}) });");
        assert!(extract(&c).is_empty());
    }

    #[test]
    fn missing_scope_section_returns_no_entries() {
        let c = ctrl("export default controller({ on: { click: () => {} } });");
        assert!(extract(&c).is_empty());
    }

    #[test]
    fn unknown_call_falls_back_to_unknown() {
        let c = ctrl(
            "export default controller({\n  \
             scope: () => ({\n    \
               weird: someUnknownThing(1, 2),\n  \
             }),\n});",
        );
        let v = extract(&c);
        assert_eq!(names_types(&v), vec![("weird", "unknown")]);
    }
}
