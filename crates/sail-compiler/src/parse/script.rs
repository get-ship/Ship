//! Script body keyword scanner.
//!
//! Walks a `<script>` body and lifts the seven Sail keyword forms into
//! the AST: `prop`, `state`, `derived`, `data`, `effect`, `event`,
//! `function`, plus `mount`/`unmount` lifecycle blocks.
//!
//! The scanner is **token-aware but expression-shallow**: it tracks
//! string and template-literal context, line/block comments, and brace
//! depth so it only matches keywords at the top level. Right-hand-side
//! expressions and function bodies are captured verbatim — the runtime
//! sees real TS source and ts-blank-space strips types at load time.
//!
//! Anything not matching a keyword form remains in `ScriptBlock::raw`
//! (which is the whole verbatim body — see `parser::build_shell`).
//! That way authors can drop `import` statements, `const` helpers, and
//! arbitrary TS at the top level alongside the reactive declarations.
//!
//! See `docs/sail/01-language-spec.md` § Script block.

use super::ast::{
    DataDecl, DerivedDecl, EffectDecl, EventDecl, FunctionDecl, PropDecl, ScriptBlock, StateDecl,
};

/// Mutate `script` in place: scan `script.raw` for keyword forms and
/// populate the typed declaration vectors.
pub fn scan(script: &mut ScriptBlock) {
    let src = script.raw.clone();
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    let mut at_stmt_start = true;

    while i < len {
        let b = bytes[i];

        // Line/block comments — skip but keep statement-start state.
        if b == b'/' && i + 1 < len {
            if bytes[i + 1] == b'/' {
                i = skip_line_comment(bytes, i);
                continue;
            }
            if bytes[i + 1] == b'*' {
                i = skip_block_comment(bytes, i);
                continue;
            }
        }

        // String / template literal — anything inside is opaque.
        if b == b'"' || b == b'\'' {
            i = skip_string(bytes, i, b);
            at_stmt_start = false;
            continue;
        }
        if b == b'`' {
            i = skip_template(bytes, i);
            at_stmt_start = false;
            continue;
        }

        // Statement separator → next non-ws position is statement-start.
        if b == b';' || b == b'\n' {
            at_stmt_start = true;
            i += 1;
            continue;
        }
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // Brace block at top level → opaque (skip balanced braces).
        if b == b'{' {
            i = skip_braces(bytes, i);
            at_stmt_start = false;
            continue;
        }

        if !at_stmt_start {
            i += 1;
            continue;
        }

        // Try a keyword match. read_word returns the identifier starting
        // at i and the position after it (or None if no identifier).
        if let Some((word, end)) = read_word(bytes, i) {
            match word {
                "prop" => {
                    if let Some((decl, next)) = parse_prop(&src, bytes, end) {
                        script.props.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "state" => {
                    if let Some((decl, next)) = parse_state(&src, bytes, end) {
                        script.state.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "derived" => {
                    if let Some((decl, next)) = parse_derived(&src, bytes, end) {
                        script.derived.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "data" => {
                    if let Some((decl, next)) = parse_data(&src, bytes, end) {
                        script.data.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "effect" => {
                    if let Some((decl, next)) = parse_effect(&src, bytes, end) {
                        script.effects.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "event" => {
                    if let Some((decl, next)) = parse_event(&src, bytes, end) {
                        script.events.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "mount" => {
                    if let Some((body, next)) = parse_lifecycle(&src, bytes, end) {
                        script.mount = Some(body);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "unmount" => {
                    if let Some((body, next)) = parse_lifecycle(&src, bytes, end) {
                        script.unmount = Some(body);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "function" | "async" => {
                    if let Some((decl, next)) = parse_function(&src, bytes, i) {
                        script.functions.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "on" => {
                    // `on form_reset {…}` / `on form_restore(state) {…}` /
                    // `on form_disabled(disabled) {…}` — typed lifecycle
                    // handlers for form-associated components.
                    if let Some((decl, next)) = parse_on_lifecycle(&src, bytes, end) {
                        script.form_lifecycle.push(decl);
                        i = next;
                        at_stmt_start = true;
                        continue;
                    }
                }
                "component" => {
                    // `component formAssociated` — element-level config.
                    // For now `formAssociated` is the only flag; future
                    // additions (e.g. `component shadow=open`) follow
                    // the same shape.
                    let after = skip_inline_ws(bytes, end);
                    if let Some((flag, next)) = read_word(bytes, after) {
                        if flag == "formAssociated" {
                            script.form_associated = true;
                            i = next;
                            at_stmt_start = true;
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        // Not a keyword — advance one byte and stop being at stmt-start.
        i += 1;
        at_stmt_start = false;
    }
}

// ── declaration parsers ─────────────────────────────────────────────────

fn parse_prop(src: &str, bytes: &[u8], start: usize) -> Option<(PropDecl, usize)> {
    let i = skip_inline_ws(bytes, start);
    let (name, mut i) = read_word(bytes, i)?;
    // Optional `?` (treat as not-required, no default change)
    if peek_byte(bytes, i) == Some(b'?') {
        i += 1;
    }
    let mut ty: Option<String> = None;
    let mut required = false;
    let mut default: Option<String> = None;

    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) == Some(b':') {
        i += 1;
        let (t, end) = read_type_until(src, bytes, i, &[b'=', b';', b'\n']);
        ty = Some(t.trim().trim_end_matches('!').trim().to_string());
        if t.trim_end().ends_with('!') {
            required = true;
        }
        i = end;
    }
    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) == Some(b'=') {
        i += 1;
        let (expr, end) = read_expr_until_stmt_end(src, bytes, i);
        default = Some(expr.trim().to_string());
        i = end;
    }

    Some((
        PropDecl {
            name: name.to_string(),
            ty,
            default,
            required,
        },
        i,
    ))
}

fn parse_state(src: &str, bytes: &[u8], start: usize) -> Option<(StateDecl, usize)> {
    let i = skip_inline_ws(bytes, start);
    let (name, mut i) = read_word(bytes, i)?;
    let mut ty: Option<String> = None;

    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) == Some(b':') {
        i += 1;
        let (t, end) = read_type_until(src, bytes, i, &[b'=']);
        ty = Some(t.trim().to_string());
        i = end;
    }
    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) != Some(b'=') {
        return None;
    }
    i += 1;
    let (initial, end) = read_expr_until_stmt_end(src, bytes, i);

    Some((
        StateDecl {
            name: name.to_string(),
            ty,
            initial: initial.trim().to_string(),
        },
        end,
    ))
}

fn parse_derived(src: &str, bytes: &[u8], start: usize) -> Option<(DerivedDecl, usize)> {
    let i = skip_inline_ws(bytes, start);
    let (name, mut i) = read_word(bytes, i)?;
    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) != Some(b'=') {
        return None;
    }
    i += 1;
    let (expr, end) = read_expr_until_stmt_end(src, bytes, i);
    Some((
        DerivedDecl {
            name: name.to_string(),
            expr: expr.trim().to_string(),
        },
        end,
    ))
}

fn parse_data(src: &str, bytes: &[u8], start: usize) -> Option<(DataDecl, usize)> {
    // `data X = expr` always means SSE-subscribed live signal — there is
    // no "static" form anymore. For a one-shot local seed, use `state`.
    let i = skip_inline_ws(bytes, start);
    let live = true;

    // Reject the legacy `data live X = expr` shape — `live` is implicit
    // now and writing it triggers a parser error so the migration is
    // immediate and visible.
    if let Some((w, _)) = read_word(bytes, i) {
        if w == "live" {
            return None;
        }
    }

    let (name, mut i) = read_word(bytes, i)?;
    let mut ty: Option<String> = None;

    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) == Some(b':') {
        i += 1;
        let (t, end) = read_type_until(src, bytes, i, &[b'=']);
        ty = Some(t.trim().to_string());
        i = end;
    }
    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) != Some(b'=') {
        return None;
    }
    i += 1;
    let (expr, end) = read_expr_until_stmt_end(src, bytes, i);

    Some((
        DataDecl {
            name: name.to_string(),
            ty,
            expr: expr.trim().to_string(),
            live,
        },
        end,
    ))
}

fn parse_effect(src: &str, bytes: &[u8], start: usize) -> Option<(EffectDecl, usize)> {
    let mut i = skip_inline_ws(bytes, start);
    let mut deps: Vec<String> = Vec::new();
    let mut once = false;

    // `effect on a, b { ... }` or `effect once { ... }` or `effect { ... }`
    if let Some((w, after)) = read_word(bytes, i) {
        match w {
            "on" => {
                i = skip_inline_ws(bytes, after);
                let (list_text, end) = read_until(src, bytes, i, b'{');
                for part in list_text.split(',') {
                    let p = part.trim();
                    if !p.is_empty() {
                        deps.push(p.to_string());
                    }
                }
                i = end;
            }
            "once" => {
                once = true;
                i = skip_inline_ws(bytes, after);
            }
            _ => {}
        }
    }

    if peek_byte(bytes, i) != Some(b'{') {
        return None;
    }
    let body_start = i + 1;
    let body_end = find_brace_close(bytes, i)?;
    let body = src[body_start..body_end].to_string();
    Some((
        EffectDecl {
            deps,
            once,
            body,
        },
        body_end + 1,
    ))
}

fn parse_event(src: &str, bytes: &[u8], start: usize) -> Option<(EventDecl, usize)> {
    let i = skip_inline_ws(bytes, start);
    let (name, mut i) = read_word(bytes, i)?;
    let mut payload_ty: Option<String> = None;
    i = skip_inline_ws(bytes, i);
    if peek_byte(bytes, i) == Some(b':') {
        i += 1;
        let (t, end) = read_type_until(src, bytes, i, &[b';', b'\n']);
        payload_ty = Some(t.trim().to_string());
        i = end;
    }
    Some((
        EventDecl {
            name: name.to_string(),
            payload_ty,
        },
        i,
    ))
}

/// Parse `on form_reset {…}` / `on form_restore(state) {…}` /
/// `on form_disabled(disabled) {…}`. The keyword `on` has already
/// been consumed; we read `form_<event>`, an optional `(payload)`
/// arg, and a `{…}` body.
fn parse_on_lifecycle(
    src: &str,
    bytes: &[u8],
    start: usize,
) -> Option<(super::ast::FormLifecycleHandler, usize)> {
    let i = skip_inline_ws(bytes, start);
    let (word, after_word) = read_word(bytes, i)?;
    let event = word.strip_prefix("form_")?.to_string();
    if !matches!(event.as_str(), "reset" | "restore" | "disabled") {
        return None;
    }
    let mut k = skip_inline_ws(bytes, after_word);

    // Optional `(payload)` parameter.
    let payload = if peek_byte(bytes, k) == Some(b'(') {
        let close = find_paren_close(bytes, k)?;
        let inner = src[k + 1..close].trim();
        let p = if inner.is_empty() { None } else { Some(inner.to_string()) };
        k = skip_inline_ws(bytes, close + 1);
        p
    } else {
        None
    };

    if peek_byte(bytes, k) != Some(b'{') {
        return None;
    }
    let body_start = k + 1;
    let body_end = find_brace_close(bytes, k)?;
    let body = src[body_start..body_end].to_string();
    Some((
        super::ast::FormLifecycleHandler { event, payload, body },
        body_end + 1,
    ))
}

fn find_paren_close(bytes: &[u8], open_at: usize) -> Option<usize> {
    debug_assert_eq!(bytes.get(open_at).copied(), Some(b'('));
    let mut depth = 0i32;
    let mut i = open_at;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn parse_lifecycle(src: &str, bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let i = skip_inline_ws(bytes, start);
    if peek_byte(bytes, i) != Some(b'{') {
        return None;
    }
    let body_start = i + 1;
    let body_end = find_brace_close(bytes, i)?;
    Some((src[body_start..body_end].to_string(), body_end + 1))
}

fn parse_function(src: &str, bytes: &[u8], start: usize) -> Option<(FunctionDecl, usize)> {
    // Capture from the keyword(s) through the closing `}` of the body.
    // Handles `function NAME(...) {...}` and `async function NAME(...) {...}`.
    let mut i = start;
    // skip leading word (function|async)
    let (w1, after1) = read_word(bytes, i)?;
    let mut name_search_from = after1;
    if w1 == "async" {
        let i2 = skip_inline_ws(bytes, after1);
        let (w2, after2) = read_word(bytes, i2)?;
        if w2 != "function" {
            return None;
        }
        name_search_from = after2;
    } else if w1 != "function" {
        return None;
    }

    let j = skip_inline_ws(bytes, name_search_from);
    let (name, mut k) = read_word(bytes, j)?;
    // walk to the body open brace, allowing args/return-type
    while k < bytes.len() && bytes[k] != b'{' {
        // skip strings to be safe
        let b = bytes[k];
        if b == b'"' || b == b'\'' {
            k = skip_string(bytes, k, b);
            continue;
        }
        if b == b'`' {
            k = skip_template(bytes, k);
            continue;
        }
        k += 1;
    }
    if k >= bytes.len() {
        return None;
    }
    let body_end = find_brace_close(bytes, k)?;
    let raw = src[start..(body_end + 1)].to_string();
    i = body_end + 1;
    Some((
        FunctionDecl {
            name: name.to_string(),
            raw,
        },
        i,
    ))
}

// ── token-aware utilities ───────────────────────────────────────────────

fn read_word(bytes: &[u8], i: usize) -> Option<(&str, usize)> {
    if i >= bytes.len() {
        return None;
    }
    let c = bytes[i];
    if !(c.is_ascii_alphabetic() || c == b'_' || c == b'$') {
        return None;
    }
    let mut j = i + 1;
    while j < bytes.len()
        && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
    {
        j += 1;
    }
    Some((std::str::from_utf8(&bytes[i..j]).ok()?, j))
}

fn peek_byte(bytes: &[u8], i: usize) -> Option<u8> {
    bytes.get(i).copied()
}

fn skip_inline_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i
}

fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    i += 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_string(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    i += 1;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if b == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_template(bytes: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if b == b'`' {
            return i + 1;
        }
        if b == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            i = skip_braces(bytes, i + 1);
            continue;
        }
        i += 1;
    }
    bytes.len()
}

/// Skip a `{ ... }` block, brace-balanced, string-aware.
fn skip_braces(bytes: &[u8], start: usize) -> usize {
    debug_assert_eq!(bytes[start], b'{');
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i = skip_line_comment(bytes, i);
                continue;
            }
            if bytes[i + 1] == b'*' {
                i = skip_block_comment(bytes, i);
                continue;
            }
        }
        if b == b'"' || b == b'\'' {
            i = skip_string(bytes, i, b);
            continue;
        }
        if b == b'`' {
            i = skip_template(bytes, i);
            continue;
        }
        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    bytes.len()
}

fn find_brace_close(bytes: &[u8], open: usize) -> Option<usize> {
    let end = skip_braces(bytes, open);
    if end == 0 || end > bytes.len() {
        return None;
    }
    if end == bytes.len() && (open >= bytes.len() || bytes[end - 1] != b'}') {
        return None;
    }
    Some(end - 1)
}

/// Read until the byte at `i` is at top level and matches one of `stops`,
/// or end of body / newline. Returns (text, end_index_at_stop).
fn read_type_until(src: &str, bytes: &[u8], start: usize, stops: &[u8]) -> (String, usize) {
    let mut i = start;
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_bracket = 0i32;
    let mut depth_angle = 0i32;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i = skip_line_comment(bytes, i);
                continue;
            }
            if bytes[i + 1] == b'*' {
                i = skip_block_comment(bytes, i);
                continue;
            }
        }
        if b == b'"' || b == b'\'' {
            i = skip_string(bytes, i, b);
            continue;
        }
        if b == b'`' {
            i = skip_template(bytes, i);
            continue;
        }
        match b {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'{' => depth_brace += 1,
            b'}' => depth_brace -= 1,
            b'[' => depth_bracket += 1,
            b']' => depth_bracket -= 1,
            b'<' => depth_angle += 1,
            b'>' => {
                // `=>` arrow lives in function-prop types; consume it as one token.
                if depth_angle > 0 {
                    depth_angle -= 1;
                } else if i > 0 && bytes[i - 1] == b'=' {
                    // arrow — consume the `>` without changing depth
                }
            }
            _ => {}
        }
        if depth_paren == 0
            && depth_brace == 0
            && depth_bracket == 0
            && depth_angle == 0
            && stops.contains(&b)
        {
            // `=>` arrow is part of a function type — don't stop on its `=`.
            if b == b'=' && peek_byte(bytes, i + 1) == Some(b'>') {
                i += 2;
                continue;
            }
            return (src[start..i].to_string(), i);
        }
        i += 1;
    }
    (src[start..].to_string(), bytes.len())
}

/// Read an expression up to the next top-level `;` or unescaped newline.
fn read_expr_until_stmt_end(src: &str, bytes: &[u8], start: usize) -> (String, usize) {
    let mut i = start;
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_bracket = 0i32;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                // Line comment. At depth 0 (no open parens/braces/
                // brackets), the comment terminates the expression so
                // `state tick = 0  // a comment` cleanly captures `0`
                // as the initial. Inside any open bracket pair it's
                // just a comment within the expression and we skip
                // past it to keep reading.
                if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0 {
                    let end = skip_line_comment(bytes, i);
                    return (src[start..i].to_string(), end);
                }
                i = skip_line_comment(bytes, i);
                continue;
            }
            if bytes[i + 1] == b'*' {
                i = skip_block_comment(bytes, i);
                continue;
            }
        }
        if b == b'"' || b == b'\'' {
            i = skip_string(bytes, i, b);
            continue;
        }
        if b == b'`' {
            i = skip_template(bytes, i);
            continue;
        }
        match b {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'{' => depth_brace += 1,
            b'}' => depth_brace -= 1,
            b'[' => depth_bracket += 1,
            b']' => depth_bracket -= 1,
            _ => {}
        }
        if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0 {
            if b == b';' {
                return (src[start..i].to_string(), i);
            }
            if b == b'\n' && expr_continues(bytes, i, src) == false {
                return (src[start..i].to_string(), i);
            }
        }
        i += 1;
    }
    (src[start..].to_string(), bytes.len())
}

/// Decide whether the expression that ended at `nl` (a newline byte
/// at depth 0) actually continues on the next non-empty line. JS-like
/// rule: an expression continues if the LAST non-whitespace byte
/// before `nl` is a binary/ternary operator (so the line can't stand
/// alone as a complete statement) OR the FIRST non-whitespace byte on
/// the next line begins with an operator/`.`/`?`/`:` (so the next
/// line is clearly a continuation, not a new statement). We also
/// continue when the next non-empty line starts with an identifier
/// that is NOT a Sail top-level keyword — but a safer heuristic is to
/// only continue on operator-prefix because identifier-prefix could
/// be a follow-on statement using the same name (rare in practice).
fn expr_continues(bytes: &[u8], nl: usize, src: &str) -> bool {
    // Look backward from nl to find the last non-whitespace byte.
    let mut j = nl;
    while j > 0 {
        j -= 1;
        let b = bytes[j];
        if b == b' ' || b == b'\t' { continue; }
        // Continuation operators at end-of-line.
        return matches!(
            b,
            b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' | b'~'
            | b'<' | b'>' | b'=' | b'!' | b'?' | b':' | b','
            | b'(' | b'[' | b'{' | b'.'
        ) || look_forward_continues(bytes, nl + 1, src);
    }
    look_forward_continues(bytes, nl + 1, src)
}

fn look_forward_continues(bytes: &[u8], from: usize, _src: &str) -> bool {
    let mut k = from;
    while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t' || bytes[k] == b'\n') {
        k += 1;
    }
    if k >= bytes.len() { return false; }
    let b = bytes[k];
    matches!(
        b,
        b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^'
        | b'<' | b'>' | b'=' | b'!' | b'?' | b':' | b','
        | b')' | b']' | b'}' | b'.'
    )
}

fn read_until(src: &str, bytes: &[u8], start: usize, stop: u8) -> (String, usize) {
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' || b == b'\'' {
            i = skip_string(bytes, i, b);
            continue;
        }
        if b == b'`' {
            i = skip_template(bytes, i);
            continue;
        }
        if b == stop {
            return (src[start..i].to_string(), i);
        }
        i += 1;
    }
    (src[start..].to_string(), bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::ScriptLang;

    fn scan_str(src: &str) -> ScriptBlock {
        let mut sb = ScriptBlock {
            lang: ScriptLang::Ts,
            raw: src.to_string(),
            props: vec![],
            state: vec![],
            derived: vec![],
            data: vec![],
            effects: vec![],
            events: vec![],
            mount: None,
            unmount: None,
            functions: vec![],
            form_associated: false,
            form_lifecycle: vec![],
        };
        scan(&mut sb);
        sb
    }

    #[test]
    fn prop_with_type_and_default() {
        let s = scan_str("prop label: string = 'Hi'");
        assert_eq!(s.props.len(), 1);
        assert_eq!(s.props[0].name, "label");
        assert_eq!(s.props[0].ty.as_deref(), Some("string"));
        assert_eq!(s.props[0].default.as_deref(), Some("'Hi'"));
        assert!(!s.props[0].required);
    }

    #[test]
    fn prop_required_bang() {
        let s = scan_str("prop required: boolean!");
        let p = &s.props[0];
        assert_eq!(p.name, "required");
        assert_eq!(p.ty.as_deref(), Some("boolean"));
        assert!(p.required);
        assert!(p.default.is_none());
    }

    #[test]
    fn prop_union_type() {
        let s = scan_str("prop variant: 'primary' | 'ghost' = 'primary'");
        let p = &s.props[0];
        assert_eq!(p.ty.as_deref(), Some("'primary' | 'ghost'"));
        assert_eq!(p.default.as_deref(), Some("'primary'"));
    }

    #[test]
    fn prop_function_type() {
        let s = scan_str("prop onSelect: (id: string) => void");
        let p = &s.props[0];
        assert_eq!(p.ty.as_deref(), Some("(id: string) => void"));
    }

    #[test]
    fn state_inferred() {
        let s = scan_str("state draft = ''");
        assert_eq!(s.state.len(), 1);
        assert_eq!(s.state[0].name, "draft");
        assert!(s.state[0].ty.is_none());
        assert_eq!(s.state[0].initial, "''");
    }

    #[test]
    fn state_typed_array() {
        let s = scan_str("state turns: Turn[] = []");
        let st = &s.state[0];
        assert_eq!(st.ty.as_deref(), Some("Turn[]"));
        assert_eq!(st.initial, "[]");
    }

    #[test]
    fn derived_basic() {
        let s = scan_str("derived total = a + b");
        assert_eq!(s.derived[0].name, "total");
        assert_eq!(s.derived[0].expr, "a + b");
    }

    #[test]
    fn data_implicit_live() {
        // `live` is implicit in the new surface — `data X = expr` always
        // means SSE-subscribed live signal. For a one-shot local seed,
        // use `state X = expr`.
        let s = scan_str("data mesh = ship.mesh.snapshot()");
        let d = &s.data[0];
        assert!(d.live);
        assert_eq!(d.name, "mesh");
        assert_eq!(d.expr, "ship.mesh.snapshot()");
    }

    #[test]
    fn data_typed() {
        let s = scan_str("data jobs: JobRecord[] = ship.jobs.list({ limit: 50 })");
        let d = &s.data[0];
        assert!(d.live);
        assert_eq!(d.ty.as_deref(), Some("JobRecord[]"));
        assert_eq!(d.expr, "ship.jobs.list({ limit: 50 })");
    }

    #[test]
    fn data_live_keyword_rejected() {
        // The legacy `data live X = expr` form is explicitly rejected
        // by the parser — `live` is implicit now and writing it is an
        // immediate signal that the file needs migration.
        let s = scan_str("data live old = ship.mesh.snapshot()");
        assert!(s.data.is_empty(), "data live should not parse to a DataDecl");
    }

    #[test]
    fn effect_on_deps() {
        let s = scan_str("effect on props.sessionId, draft {\n  console.log('changed')\n}");
        let e = &s.effects[0];
        assert_eq!(e.deps, vec!["props.sessionId", "draft"]);
        assert!(!e.once);
        assert!(e.body.contains("console.log"));
    }

    #[test]
    fn effect_once() {
        let s = scan_str("effect once { setup() }");
        let e = &s.effects[0];
        assert!(e.once);
        assert!(e.deps.is_empty());
        assert!(e.body.contains("setup()"));
    }

    #[test]
    fn effect_bare() {
        let s = scan_str("effect { ping() }");
        let e = &s.effects[0];
        assert!(!e.once);
        assert!(e.deps.is_empty());
    }

    #[test]
    fn event_with_payload() {
        let s = scan_str("event select: { id: string, item: Item }");
        let e = &s.events[0];
        assert_eq!(e.name, "select");
        assert_eq!(e.payload_ty.as_deref(), Some("{ id: string, item: Item }"));
    }

    #[test]
    fn event_no_payload() {
        let s = scan_str("event close");
        assert_eq!(s.events[0].name, "close");
        assert!(s.events[0].payload_ty.is_none());
    }

    #[test]
    fn function_sync() {
        let s = scan_str("function send() {\n  doStuff()\n}");
        assert_eq!(s.functions.len(), 1);
        assert_eq!(s.functions[0].name, "send");
        assert!(s.functions[0].raw.starts_with("function send"));
        assert!(s.functions[0].raw.ends_with('}'));
    }

    #[test]
    fn function_async() {
        let s = scan_str("async function refresh() {\n  await x()\n}");
        assert_eq!(s.functions[0].name, "refresh");
        assert!(s.functions[0].raw.starts_with("async function"));
    }

    #[test]
    fn lifecycle_mount_unmount() {
        let s = scan_str("mount {\n  el.focus()\n}\nunmount { teardown() }");
        assert!(s.mount.as_deref().unwrap().contains("el.focus()"));
        assert!(s.unmount.as_deref().unwrap().contains("teardown()"));
    }

    #[test]
    fn ignores_keywords_inside_strings() {
        let s = scan_str("const msg = 'state x = 1; prop y = 2'");
        assert!(s.state.is_empty());
        assert!(s.props.is_empty());
    }

    #[test]
    fn ignores_keywords_inside_comments() {
        let s = scan_str("// state foo = 1\n/* prop bar = 2 */\n");
        assert!(s.state.is_empty());
        assert!(s.props.is_empty());
    }

    #[test]
    fn ignores_keywords_in_function_body() {
        let s = scan_str("function f() { const state = 1; let prop = 2 }");
        assert_eq!(s.functions.len(), 1);
        assert!(s.state.is_empty());
        assert!(s.props.is_empty());
    }

    #[test]
    fn template_literal_with_braces_safe() {
        let s = scan_str("state msg = `hello ${1 + 2} state x`");
        assert_eq!(s.state.len(), 1);
        assert_eq!(s.state[0].name, "msg");
    }

    #[test]
    fn multiple_decls() {
        let src = r#"
prop label: string = 'a'
state count = 0
derived doubled = count * 2
data mesh = ship.mesh.snapshot()
event select: { id: string }
function go() { count++ }
mount { init() }
"#;
        let s = scan_str(src);
        assert_eq!(s.props.len(), 1);
        assert_eq!(s.state.len(), 1);
        assert_eq!(s.derived.len(), 1);
        assert_eq!(s.data.len(), 1);
        assert_eq!(s.events.len(), 1);
        assert_eq!(s.functions.len(), 1);
        assert!(s.mount.is_some());
    }

    #[test]
    fn component_form_associated_keyword() {
        let s = scan_str("component formAssociated\nstate x = 0");
        assert!(s.form_associated);
        assert_eq!(s.state.len(), 1);
        assert_eq!(s.state[0].name, "x");
    }

    #[test]
    fn component_keyword_default_is_false() {
        let s = scan_str("state x = 0");
        assert!(!s.form_associated);
    }

    #[test]
    fn imports_and_consts_are_passed_through_silently() {
        // These shouldn't produce keyword decls; raw remains intact for
        // ts-blank-space loading at runtime.
        let s = scan_str("import { x } from './x'\nconst y = 1\nstate z = y");
        assert_eq!(s.state.len(), 1);
        assert_eq!(s.state[0].name, "z");
    }
}
