//! Generic byte-level scanning primitives used by the scope-factory
//! extractor and its type-inference heuristic. These are string-, comment-,
//! and bracket-aware so callers don't match inside string literals or walk
//! past the end of a balanced group. No scope-factory domain knowledge lives
//! here — they're lexical leaves.
//!
//! (Sibling copies of some of these exist elsewhere in the compiler with
//! diverged signatures; consolidating them crate-wide is tracked separately.)

pub(super) fn skip_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
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

pub(super) fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
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

pub(super) fn next_top_separator(bytes: &[u8], start: usize, end: usize) -> usize {
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
            // A `<` in a generic position (`signal<Record<A, B>>(…)`) opens a
            // type-argument block whose commas are NOT entry separators. Without
            // skipping it, `signal<Record<Section, boolean>>(…)` truncated at the
            // inner comma and the whole entry degraded to bare `unknown`. Only
            // treat `<` as a generic when it follows an identifier char (`Foo<`)
            // so we don't misread a stray comparison.
            b'<' if k > start && is_ident_cont(bytes[k - 1]) => {
                k = skip_angle(bytes, k, end);
            }
            b',' | b'}' => return k,
            _ => k += 1,
        }
    }
    end
}

/// Skip a balanced `<…>` generic block starting at `bytes[i] == b'<'`.
/// Returns the index just past the matching `>`, or `i + 1` if it can't
/// balance (so the caller still makes progress). Tracks `(`/`{`/`[` and
/// string literals inside the type args, and ignores `=>` / `<=` / `>=`
/// so an arrow or comparison inside the block doesn't unbalance the count.
pub(super) fn skip_angle(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end || bytes[i] != b'<' {
        return (i + 1).min(end);
    }
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
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            // `=>` is an arrow, not a closing angle.
            b'=' if k + 1 < end && bytes[k + 1] == b'>' => { k += 2; }
            // `<=` / `>=` are comparisons, not bracket motion.
            b'<' if k + 1 < end && bytes[k + 1] == b'=' => { k += 2; }
            b'>' if k + 1 < end && bytes[k + 1] == b'=' => { k += 2; }
            b'<' => { depth += 1; k += 1; }
            b'>' => {
                depth -= 1;
                k += 1;
                if depth == 0 {
                    return k;
                }
            }
            _ => k += 1,
        }
    }
    (i + 1).min(end)
}

pub(super) fn advance_past_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let stop = next_top_separator(bytes, start, end);
    if stop < end && bytes[stop] == b',' {
        stop + 1
    } else {
        stop
    }
}

#[inline]
pub(super) fn is_ident_start(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic()
}

#[inline]
pub(super) fn is_ident_cont(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}
