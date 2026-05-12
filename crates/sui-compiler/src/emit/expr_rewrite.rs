//! Rewrite a template-expression source string so every free
//! identifier is qualified with `$scope.`.
//!
//! The compiler emits `() => $scope.count()` style closures directly
//! into the hydrate body — no `with($scope)` wrapper, no `new Function`
//! eval. Locals introduced by arrow params or destructuring are left
//! alone; JS / browser builtins (`Math`, `document`, …) are also left
//! alone since they resolve from the global lexical scope.
//!
//! The walker mirrors `check::scope_bound_expr` so the `$scope.foo`
//! qualification matches the deps the bindings registry tracks.

/// JS / browser globals + framework-known idents that resolve outside
/// `$scope`. Kept in sync with `check::scope_bound::BUILTINS` — any
/// identifier the validator accepts as a builtin must be left
/// unqualified by the rewriter.
const BUILTINS: &[&str] = &[
    // Setup context propagated onto $scope by the compiled module.
    // `ship`, `route`, `navigate` are seeded into $scope for views, so
    // they DO need qualification (handled by the default branch).
    // `ctx` is only ever a parent-scope ident in handlers — leave bare.
    "ctx",
    // JS standard objects.
    "Math", "JSON", "Object", "Array", "String", "Number", "Boolean", "Date",
    "Symbol", "Map", "Set", "WeakMap", "WeakSet", "Error", "RegExp",
    "Promise", "BigInt", "ArrayBuffer", "DataView",
    "Int8Array", "Uint8Array", "Uint8ClampedArray",
    "Int16Array", "Uint16Array", "Int32Array", "Uint32Array",
    "Float32Array", "Float64Array",
    // Standard literals.
    "null", "undefined", "true", "false", "NaN", "Infinity",
    // Standard fns.
    "parseInt", "parseFloat", "isNaN", "isFinite", "encodeURIComponent",
    "decodeURIComponent", "encodeURI", "decodeURI",
    // Browser globals commonly used in handlers.
    "document", "window", "console", "globalThis",
    // Runtime helpers imported by the compiled module.
    "unwrap", "unwrapOr",
];

const KEYWORDS: &[&str] = &[
    "if", "else", "return", "let", "const", "var", "for", "while", "do",
    "switch", "case", "break", "continue", "typeof", "instanceof", "in",
    "of", "new", "async", "await", "yield", "function", "class", "this",
    "super", "void", "delete",
];

/// Rewrite `src` so free identifiers are qualified with `$scope.`.
///
/// `extra_locals` is a list of identifiers already bound in the
/// surrounding emit context (e.g. a `{#for item}` introduces `item`
/// inside the body's expressions). These are treated as locals and
/// left bare — the compiled `for` body shadows `$scope` with a child
/// that carries the iteration var, so a bare `item` is also reachable
/// via `$scope.item`, but bare reads are smaller and equivalently
/// correct.
pub fn rewrite_for_scope(src: &str, extra_locals: &[&str]) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len() + 16);
    let mut i = 0;
    let mut prev_dot = false;
    let mut in_str: Option<u8> = None;
    // Stack of locals introduced by arrow params we've stepped past in
    // this expression. We never pop — once introduced inside this
    // expression, the binding shadows $scope for the rest of the walk.
    // (Mismatch with strict scoping is acceptable: false negatives only
    // happen when an inner identifier shadows then a same-name outer
    // re-uses the ident — which doesn't happen in template expressions.)
    let mut locals: Vec<String> = extra_locals.iter().map(|s| s.to_string()).collect();
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            // Forward a single source char (1–4 bytes) to the output.
            // Previous impl used `c as char` which silently truncated
            // multi-byte UTF-8 sequences into Latin-1 chars.
            let ch_end = next_char_boundary(src, i);
            out.push_str(&src[i..ch_end]);
            if c == b'\\' && ch_end < bytes.len() {
                // Forward the next char verbatim — covers `\n`, `\"`,
                // and multi-byte escapes like `\é`.
                let esc_end = next_char_boundary(src, ch_end);
                out.push_str(&src[ch_end..esc_end]);
                i = esc_end;
                continue;
            }
            if c == q {
                in_str = None;
            } else if q == b'`' && c == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                // Template literal interpolation — recurse on the inner
                // expression so its free idents get qualified too.
                out.push('{');
                let close = skip_balanced(bytes, i + 1, bytes.len());
                // Snap the slice ends to char boundaries — `skip_balanced`
                // works on bytes and can land mid-multi-byte if the input
                // contains decoded HTML entities (`§`, `°`, etc.).
                let inner_start = (i + 2).min(src.len());
                let inner_end = close.saturating_sub(1).min(src.len());
                let inner_start = floor_char_boundary(src, inner_start);
                let inner_end = floor_char_boundary(src, inner_end);
                let inner = if inner_start <= inner_end {
                    &src[inner_start..inner_end]
                } else {
                    ""
                };
                let rewritten = rewrite_for_scope(
                    inner,
                    &locals.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                );
                out.push_str(&rewritten);
                out.push('}');
                in_str = Some(b'`');
                i = close;
                continue;
            }
            i = ch_end;
            continue;
        }
        // Outside a string literal, all language-significant chars are
        // ASCII (quotes, parens, dots, identifiers). Non-ASCII bytes
        // outside a string literal aren't valid JS, but if they appear
        // we still forward them as full chars to avoid mangling them.
        if c == b'"' || c == b'\'' || c == b'`' {
            in_str = Some(c);
            out.push(c as char);
            i += 1;
            prev_dot = false;
            continue;
        }
        if c == b'.' {
            // Member access. The next ident is a property name, not a
            // free ident. But `?.` short-circuit and `..` are also
            // possible — we only set `prev_dot` for a single `.`.
            out.push('.');
            prev_dot = true;
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            out.push(c as char);
            i += 1;
            continue;
        }
        if !is_ident_start(c) {
            // Maybe an arrow-param list: `(a, b) => …`. Probe for the
            // arrow and harvest params if so.
            if c == b'('
                && let Some(after) = peek_arrow_params(src, i, &mut locals)
            {
                out.push_str(&src[i..after]);
                i = after;
                prev_dot = false;
                continue;
            }
            // Forward a single char — preserves multi-byte non-ASCII
            // (which shouldn't reach here in valid JS, but compiler
            // robustness > strict spec adherence).
            let ch_end = next_char_boundary(src, i);
            out.push_str(&src[i..ch_end]);
            prev_dot = false;
            i = ch_end;
            continue;
        }
        // Read the identifier.
        let start = i;
        while i < bytes.len() && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = &src[start..i];
        if KEYWORDS.contains(&word) {
            out.push_str(word);
            prev_dot = false;
            continue;
        }
        // `x => …` — single-ident arrow. The ident is a local param.
        if peek_arrow_after(src, i) {
            locals.push(word.to_string());
            out.push_str(word);
            prev_dot = false;
            continue;
        }
        if prev_dot {
            // Member name — bare.
            out.push_str(word);
            prev_dot = false;
            continue;
        }
        // `{ key: value, ... }` — object-literal shorthand. We can't
        // distinguish `{ key: $scope.key }` from `{ key: scope_var }`
        // without a real parser. Conservative: qualify the value side
        // (after `:`); leave keys alone. Keys are detected by the
        // following `:` (and not `::`).
        if peek_colon(src, i) && is_object_key_position(&out, src, start) {
            out.push_str(word);
            prev_dot = false;
            continue;
        }
        if BUILTINS.contains(&word) || locals.iter().any(|s| s == word) {
            out.push_str(word);
            prev_dot = false;
            continue;
        }
        out.push_str("$scope.");
        out.push_str(word);
        prev_dot = false;
    }
    out
}

fn peek_arrow_after(src: &str, i: usize) -> bool {
    let bytes = src.as_bytes();
    let mut k = i;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    k + 1 < bytes.len() && bytes[k] == b'=' && bytes[k + 1] == b'>'
}

fn peek_colon(src: &str, i: usize) -> bool {
    let bytes = src.as_bytes();
    let mut k = i;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    k < bytes.len() && bytes[k] == b':'
        && (k + 1 >= bytes.len() || bytes[k + 1] != b':')
}

/// Heuristic: `out` ends with the last byte we already emitted before
/// reading `word` at `start`. Object-key position is when the
/// preceding non-trivia char is `{` or `,`. This is conservative —
/// `case X:` (a label) also qualifies as "after non-trivia"; the
/// scope-bound check rejects labels in template expressions so we
/// don't have to handle them.
fn is_object_key_position(out: &str, _src: &str, _start: usize) -> bool {
    let trimmed = out.trim_end_matches(|c: char| c.is_ascii_whitespace());
    trimmed.ends_with('{') || trimmed.ends_with(',')
}

/// Probe a `(` for an arrow param list. If `(a, b) =>` follows,
/// harvest the params into `locals` and return the offset just past
/// the `)`. Returns `None` for non-arrow `(` (function call /
/// grouping).
fn peek_arrow_params(src: &str, i: usize, locals: &mut Vec<String>) -> Option<usize> {
    let bytes = src.as_bytes();
    let close = skip_balanced(bytes, i, bytes.len());
    if close <= i || close > bytes.len() {
        return None;
    }
    let mut k = close;
    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
        k += 1;
    }
    let is_arrow = k + 1 < bytes.len() && bytes[k] == b'=' && bytes[k + 1] == b'>';
    if !is_arrow {
        return None;
    }
    // Harvest top-level idents inside the parens, treating `,` as
    // separators and skipping nested groupings (defaults, destructure).
    let inner_start = i + 1;
    let inner_end = close.saturating_sub(1);
    harvest_param_idents(src, inner_start, inner_end, locals);
    Some(close)
}

fn harvest_param_idents(src: &str, start: usize, end: usize, into: &mut Vec<String>) {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut paren = 0i32;
    let mut in_str: Option<u8> = None;
    let mut skip_until_separator = false;
    while i < end {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' { i += 2; continue; }
            if c == q { in_str = None; }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); i += 1; }
            b'(' | b'<' => { paren += 1; i += 1; }
            b')' | b'>' => { paren -= 1; i += 1; }
            b':' | b'=' if paren == 0 => { skip_until_separator = true; i += 1; }
            b',' if paren == 0 => { skip_until_separator = false; i += 1; }
            b'{' | b'[' if !skip_until_separator => {
                let close = skip_balanced(bytes, i, end);
                harvest_destructure_idents(src, i + 1, close.saturating_sub(1), into);
                i = close;
            }
            _ if !skip_until_separator && is_ident_start(c) => {
                let s = i;
                while i < end && is_ident_cont(bytes[i]) { i += 1; }
                let w = &src[s..i];
                if !KEYWORDS.contains(&w) {
                    into.push(w.to_string());
                }
                skip_until_separator = true;
            }
            _ => i += 1,
        }
    }
}

fn harvest_destructure_idents(src: &str, start: usize, end: usize, into: &mut Vec<String>) {
    let bytes = src.as_bytes();
    let mut i = start;
    let mut last_ident: Option<String> = None;
    let mut after_colon = false;
    while i < end {
        let c = bytes[i];
        if c.is_ascii_whitespace() { i += 1; continue; }
        if c == b':' { after_colon = true; i += 1; continue; }
        if c == b',' {
            if !after_colon
                && let Some(w) = last_ident.take() {
                    into.push(w);
                }
            after_colon = false;
            last_ident = None;
            i += 1;
            continue;
        }
        if c == b'{' || c == b'[' {
            let close = skip_balanced(bytes, i, end);
            harvest_destructure_idents(src, i + 1, close.saturating_sub(1), into);
            i = close;
            after_colon = false;
            last_ident = None;
            continue;
        }
        if is_ident_start(c) {
            let s = i;
            while i < end && is_ident_cont(bytes[i]) { i += 1; }
            let w = src[s..i].to_string();
            if after_colon {
                into.push(w);
                last_ident = None;
                after_colon = false;
            } else {
                last_ident = Some(w);
            }
            continue;
        }
        i += 1;
    }
    if !after_colon
        && let Some(w) = last_ident {
            into.push(w);
        }
}

fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end { return end; }
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
            if c == b'\\' { k += 2; continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            c if c == open => { depth += 1; k += 1; }
            c if c == close => {
                depth -= 1;
                k += 1;
                if depth == 0 { return k; }
            }
            _ => k += 1,
        }
    }
    end
}

/// Largest `j <= i` such that `j` is a char boundary in `s`. Used by
/// the template-literal recursion above to snap byte indices that
/// `skip_balanced` may have landed inside a multi-byte UTF-8 char
/// (e.g. when the source contains decoded HTML entities).
#[inline]
fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Byte position of the char boundary immediately after `i`. Returns
/// `s.len()` if `i` is at or past the end. Used to forward a single
/// char (ASCII or multi-byte) verbatim without the `as char` truncation
/// that silently corrupts non-ASCII bytes.
#[inline]
fn next_char_boundary(s: &str, i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    let mut j = i + 1;
    while j < s.len() && !s.is_char_boundary(j) {
        j += 1;
    }
    j
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

    fn rw(s: &str) -> String {
        rewrite_for_scope(s, &[])
    }

    #[test]
    fn simple_call_qualified() {
        assert_eq!(rw("count()"), "$scope.count()");
    }

    #[test]
    fn property_access_left_alone() {
        assert_eq!(rw("user.name"), "$scope.user.name");
    }

    #[test]
    fn nested_member_access() {
        assert_eq!(rw("user.profile.name"), "$scope.user.profile.name");
    }

    #[test]
    fn arrow_param_local() {
        assert_eq!(rw("(e) => e.target.value"), "(e) => e.target.value");
    }

    #[test]
    fn single_arrow_local() {
        assert_eq!(rw("x => x + 1"), "x => x + 1");
    }

    #[test]
    fn handler_setter_call() {
        assert_eq!(
            rw("() => count.set(count() + 1)"),
            "() => $scope.count.set($scope.count() + 1)"
        );
    }

    #[test]
    fn builtin_left_bare() {
        assert_eq!(rw("Math.max(0, count())"), "Math.max(0, $scope.count())");
    }

    #[test]
    fn keyword_left_bare() {
        assert_eq!(rw("typeof count"), "typeof $scope.count");
    }

    #[test]
    fn string_content_untouched() {
        assert_eq!(rw("\"count\""), "\"count\"");
        assert_eq!(rw("'foo' + count()"), "'foo' + $scope.count()");
    }

    #[test]
    fn template_literal_inner_qualified() {
        assert_eq!(
            rw("`hello ${name}!`"),
            "`hello ${$scope.name}!`"
        );
    }

    #[test]
    fn object_literal_keys_left_alone() {
        assert_eq!(rw("{ a: count(), b: 1 }"), "{ a: $scope.count(), b: 1 }");
    }

    #[test]
    fn nested_destructure_arrow() {
        assert_eq!(
            rw("({ a, b }) => a + b + count"),
            "({ a, b }) => a + b + $scope.count"
        );
    }

    #[test]
    fn for_iteration_local() {
        assert_eq!(
            rewrite_for_scope("item.name + count()", &["item"]),
            "item.name + $scope.count()"
        );
    }

    #[test]
    fn nullish_coalesce() {
        assert_eq!(rw("x ?? defaultValue"), "$scope.x ?? $scope.defaultValue");
    }

    #[test]
    fn ternary() {
        assert_eq!(
            rw("count() > 0 ? \"yes\" : \"no\""),
            "$scope.count() > 0 ? \"yes\" : \"no\""
        );
    }

    #[test]
    fn handler_with_event_param() {
        assert_eq!(
            rw("(e) => name.set(e.target.value)"),
            "(e) => $scope.name.set(e.target.value)"
        );
    }

    #[test]
    fn console_log_left_bare() {
        assert_eq!(
            rw("() => console.log(name)"),
            "() => console.log($scope.name)"
        );
    }
}

#[cfg(test)]
mod multibyte_tests {
    use super::*;

    #[test]
    fn multibyte_in_string_literal_preserved() {
        // Pre-fix this emitted Latin-1 garbage for the non-ASCII bytes.
        let out = rewrite_for_scope(r#"foo("hello °, world ™")"#, &[]);
        assert!(out.contains('°'), "° preserved in string output, got: {out}");
        assert!(out.contains('™'), "™ preserved in string output, got: {out}");
    }

    #[test]
    fn multibyte_in_template_literal_preserved() {
        let out = rewrite_for_scope(r#"`pre § post`"#, &[]);
        assert!(out.contains('§'), "§ preserved in template literal, got: {out}");
    }

    #[test]
    fn escaped_multibyte_in_string_preserved() {
        // `\°` — backslash followed by multi-byte char. Previously the
        // multi-byte lead byte was cast to char (Latin-1), corrupting it.
        let out = rewrite_for_scope("foo(\"\\°\")", &[]);
        assert!(out.contains('°'), "escaped ° preserved, got: {out}");
    }
}
