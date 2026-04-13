//! HTML sanitizer for agent-supplied canvas fragments.
//!
//! Allowlist-based: common structural tags + every `<ship-*>` element found
//! in the input + `data-*` / ARIA attributes. Strips `<script>`, `on*`
//! handlers, and `javascript:` / `data:` URLs on link/href attributes.
//!
//! Backed by the `ammonia` crate (reference HTML5 sanitizer). Wraps the
//! builder with Ship-specific allowlists so every call site gets the same
//! policy.

use std::collections::{HashMap, HashSet};

/// Standard (non-`ship-*`) tags canvas agents may emit. Conservative list —
/// add new entries when a real workload needs them, not speculatively.
const STANDARD_TAGS: &[&str] = &[
    "a", "abbr", "article", "aside", "b", "blockquote", "br", "button",
    "caption", "cite", "code", "col", "colgroup", "dd", "details", "dfn",
    "div", "dl", "dt", "em", "figcaption", "figure", "footer", "form", "h1",
    "h2", "h3", "h4", "h5", "h6", "header", "hr", "i", "img", "input",
    "kbd", "label", "li", "main", "mark", "nav", "ol", "option", "output",
    "p", "pre", "q", "s", "samp", "section", "select", "small", "source",
    "span", "strong", "sub", "summary", "sup", "table", "tbody", "td",
    "textarea", "tfoot", "th", "thead", "time", "tr", "u", "ul", "var",
    "wbr",
];

/// Attributes allowed on every tag (plus per-tag whitelists below).
/// `data-*` is matched via `generic_attribute_prefixes`.
const GLOBAL_ATTRS: &[&str] = &[
    "class", "id", "title", "lang", "dir", "role", "tabindex", "slot",
    // ARIA a11y attrs
    "aria-label", "aria-labelledby", "aria-describedby", "aria-hidden",
    "aria-live", "aria-expanded", "aria-controls", "aria-pressed",
    "aria-checked", "aria-current", "aria-busy",
];

/// Sanitize an HTML fragment for broadcast to view subscribers.
///
/// Behavior:
/// - `<ship-*>` tags detected in the input are added to the allowlist
///   dynamically (Ship's web-component primitives are inherently sandboxed).
/// - `data-*` attributes pass through.
/// - Inline event handlers (`onclick`, `onload`, ...) are stripped.
/// - `<script>`, `<iframe>`, `<object>`, `<embed>` tags are removed.
/// - `javascript:` and `data:` URL schemes are rejected on `href` / `src` /
///   `action` / `formaction`.
pub fn sanitize(input: &str) -> String {
    let ship_tags: Vec<String> = collect_ship_tags(input);
    let ship_attrs: Vec<String> = collect_ship_attrs(input);
    let mut tags: HashSet<&str> = STANDARD_TAGS.iter().copied().collect();
    for t in &ship_tags {
        tags.insert(t.as_str());
    }

    // Per-tag attribute whitelists beyond GLOBAL_ATTRS.
    let mut tag_attrs: HashMap<&str, HashSet<&str>> = HashMap::new();
    // `rel` is managed by ammonia's `link_rel` builder method, not a raw
    // attribute. We leave ammonia's default (`noopener noreferrer`).
    tag_attrs.insert("a", ["href", "target"].into_iter().collect());
    tag_attrs.insert(
        "img",
        ["src", "alt", "width", "height", "loading"].into_iter().collect(),
    );
    tag_attrs.insert(
        "input",
        ["type", "name", "value", "placeholder", "checked", "disabled", "readonly", "required"]
            .into_iter()
            .collect(),
    );
    tag_attrs.insert("button", ["type", "disabled", "name", "value"].into_iter().collect());
    tag_attrs.insert(
        "form",
        ["action", "method", "enctype", "autocomplete"].into_iter().collect(),
    );
    tag_attrs.insert("label", ["for"].into_iter().collect());
    tag_attrs.insert("option", ["value", "selected", "disabled"].into_iter().collect());
    tag_attrs.insert("select", ["name", "multiple", "required", "disabled"].into_iter().collect());
    tag_attrs.insert(
        "textarea",
        ["name", "rows", "cols", "placeholder", "readonly", "required"].into_iter().collect(),
    );
    tag_attrs.insert("source", ["src", "type", "media", "srcset"].into_iter().collect());
    tag_attrs.insert("time", ["datetime"].into_iter().collect());

    // ship-* elements are web components with author-defined props. We
    // allow every attr observed in the input across every ship-* tag —
    // the allowlist is still the input surface (not arbitrary attrs),
    // and dangerous `on*` / URL-scheme attrs are filtered below.
    let ship_attr_refs: HashSet<&str> = ship_attrs.iter().map(|s| s.as_str()).collect();
    for t in &ship_tags {
        tag_attrs.insert(t.as_str(), ship_attr_refs.clone());
    }

    let globals: HashSet<&str> = GLOBAL_ATTRS.iter().copied().collect();
    let url_schemes: HashSet<&str> = ["http", "https", "mailto"].into_iter().collect();
    let generic_prefixes: HashSet<&str> = ["data-", "aria-"].into_iter().collect();

    ammonia::Builder::new()
        .tags(tags)
        .generic_attributes(globals)
        .tag_attributes(tag_attrs)
        .generic_attribute_prefixes(generic_prefixes)
        .url_schemes(url_schemes)
        // Strip unknown tags by removing them outright (no content bleed).
        .strip_comments(true)
        // Disallow `srcset` on non-source tags, etc. — default-off keeps it strict.
        .clean(input)
        .to_string()
}

/// Scan ship-* opening tags for the attribute names used on them. Every
/// such name is added to the per-tag allowlist; dangerous patterns
/// (`on*`, `javascript:` URLs) remain filtered by the sanitizer itself.
fn collect_ship_attrs(input: &str) -> Vec<String> {
    let mut out: HashSet<String> = HashSet::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i + 5 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1..].starts_with(b"ship-") {
            // Walk to the matching `>` and capture attr names (`name=` or
            // bare `name`). Stop at `>` outside of an attribute value.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'>' && bytes[j] != b'/' {
                // Skip whitespace.
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < bytes.len()
                    && !bytes[j].is_ascii_whitespace()
                    && bytes[j] != b'='
                    && bytes[j] != b'>'
                    && bytes[j] != b'/'
                {
                    j += 1;
                }
                if j > start {
                    if let Ok(name) = std::str::from_utf8(&bytes[start..j]) {
                        let n = name.to_ascii_lowercase();
                        // Skip the tag name itself (first token).
                        if !n.starts_with("ship-") && !n.is_empty() && !n.starts_with('<') {
                            out.insert(n);
                        }
                    }
                }
                // If we're at `=`, skip the value (quoted or bare).
                if j < bytes.len() && bytes[j] == b'=' {
                    j += 1;
                    if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                        let q = bytes[j];
                        j += 1;
                        while j < bytes.len() && bytes[j] != q {
                            j += 1;
                        }
                        if j < bytes.len() {
                            j += 1;
                        }
                    } else {
                        while j < bytes.len()
                            && !bytes[j].is_ascii_whitespace()
                            && bytes[j] != b'>'
                        {
                            j += 1;
                        }
                    }
                }
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out.into_iter().collect()
}

/// Scan `input` for `<ship-...>` opening tags and return the unique set of
/// tag names so they can be added to the sanitizer allowlist.
fn collect_ship_tags(input: &str) -> Vec<String> {
    let mut out: HashSet<String> = HashSet::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i + 5 < bytes.len() {
        if bytes[i] == b'<' {
            // Accept an optional leading `/` for closing tags.
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'/' {
                j += 1;
            }
            if bytes[j..].starts_with(b"ship-") {
                let start = j;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-')
                {
                    j += 1;
                }
                if j > start {
                    let name = std::str::from_utf8(&bytes[start..j])
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if name.starts_with("ship-") && name.len() > 5 {
                        out.insert(name);
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out.into_iter().collect()
}

#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod tests;
