//! AST types for the Sail parser.
//!
//! Mirrors the language spec at `.artifacts/specs/sail/01-language-spec.md`.

use crate::Span;

#[derive(Debug, Clone, Default)]
pub struct SailFile {
    pub script: Option<ScriptBlock>,
    pub template: Option<Template>,
    pub style: Option<StyleBlock>,
    pub source_map: SourceMap,
}

/// Holds the original source plus a precomputed line-start index so any
/// byte offset can be resolved to a 1-based `(line, col)` position in
/// O(log n). Built once at parse time and shared by both the parser
/// (which populates AST spans) and analyzers that scan raw block bodies
/// (style tokens, listener leak detector).
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// Original source for span resolution.
    pub source: String,
    /// Byte offset where each line starts. `line_starts[0]` is always 0;
    /// for an N-line source there are N entries (one per line) — every
    /// `\n` advances to the next start.
    line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { source: source.to_string(), line_starts }
    }

    /// Convert a byte offset in the source to a 1-based `(line, col)`.
    /// Columns are byte columns; ASCII source — every Sail keyword and
    /// HTML element name is ASCII — round-trips cleanly. Multi-byte
    /// characters in string literals or interpolations contribute their
    /// byte width to the column.
    pub fn position(&self, offset: usize) -> (u32, u32) {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        let col = offset.saturating_sub(line_start);
        ((line_idx as u32) + 1, (col as u32) + 1)
    }

    /// Build a [`Span`] for the half-open byte range `[start, end)`.
    /// `end` is exclusive — a single-character span at offset `o` is
    /// `(o, o + 1)`. The returned span carries an empty `file` since
    /// the parser doesn't know the file path; LSP and CLI consumers
    /// inject the path at render time.
    pub fn span(&self, start: usize, end: usize) -> Span {
        let (line, col) = self.position(start);
        let (end_line, end_col) = self.position(end);
        Span { file: String::new(), line, col, end_line, end_col }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    /// `let`, `const`, or `var`
    pub kind: String,
    pub name: String,
    pub ty: Option<String>,
    pub initializer: Option<String>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct ScriptBlock {
    pub lang: ScriptLang,
    pub raw: String,                       // verbatim, for type-stripping
    /// Byte offset of `raw` within the original source. Analyzers that
    /// scan `raw` (e.g. cleanup_pattern's listener leak detector) add
    /// this to a body-relative offset to get an absolute source offset
    /// suitable for `SourceMap::position`.
    pub body_byte_offset: usize,
    pub props: Vec<PropDecl>,
    pub state: Vec<StateDecl>,
    pub derived: Vec<DerivedDecl>,
    pub data: Vec<DataDecl>,
    pub effects: Vec<EffectDecl>,
    pub events: Vec<EventDecl>,
    pub mount: Option<String>,
    pub unmount: Option<String>,
    pub functions: Vec<FunctionDecl>,
    pub module_decls: Vec<ModuleDecl>,
    /// `component formAssociated` declaration. When true, the emitted
    /// custom element class declares `static formAssociated = true`,
    /// the runtime calls `attachInternals()` at construction, and the
    /// resulting `ElementInternals` is exposed as `internals` in the
    /// script's scope (mount block, effects, functions).
    pub form_associated: bool,
    /// `on form_reset {…}`, `on form_restore(state) {…}`,
    /// `on form_disabled(disabled) {…}` — typed top-level event blocks
    /// for form-associated lifecycle. The compiler turns each into an
    /// `el.addEventListener('sail:form-<event>', …)` plus a cleanup,
    /// so authors write the lifecycle handler the same way every time
    /// without juggling event-string lookups.
    pub form_lifecycle: Vec<FormLifecycleHandler>,
}

#[derive(Debug, Clone)]
pub struct FormLifecycleHandler {
    /// Lifecycle hook: "reset", "restore", or "disabled".
    pub event: String,
    /// Optional payload binding name pulled from `e.detail.<event>`.
    /// `on form_restore(state) {…}` → `Some("state")`.
    pub payload: Option<String>,
    /// Verbatim block body.
    pub body: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLang {
    Js,
    Ts,
}

#[derive(Debug, Clone)]
pub struct PropDecl {
    pub name: String,
    pub ty: Option<String>,                // verbatim TS source
    pub default: Option<String>,           // verbatim JS expression
    pub required: bool,                    // `T!` syntax → true
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    pub ty: Option<String>,
    pub initial: String,                   // verbatim JS
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct DerivedDecl {
    pub name: String,
    pub expr: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    pub ty: Option<String>,
    pub expr: String,                      // `ship.x.y(args)` etc.
    pub live: bool,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub deps: Vec<String>,                 // `effect on a, b { ... }` → ["a","b"]
    pub once: bool,                        // `effect once { ... }`
    pub body: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct EventDecl {
    pub name: String,
    pub payload_ty: Option<String>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub raw: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Default)]
pub struct Template {
    pub root: Vec<Node>,
    /// `<extends src="…" />` declaration at the top of the template.
    /// When present, the compiler reads the parent file, walks its
    /// template, and replaces each `<block name="X">` with the matching
    /// `<fill name="X">` from this file. Resolved before emit.
    pub extends: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<Node>,
        span: Option<Span>,
    },
    /// Sail or third-party custom element. Distinguished from Element
    /// by the registry; parser doesn't know yet — analyzer resolves.
    Component {
        tag: String,
        props: Vec<PropBinding>,
        events: Vec<EventBinding>,
        slots: Vec<SlotFill>,
        children: Vec<Node>,
        span: Option<Span>,
    },
    Text(String),
    Interpolation(Expr),
    If {
        cond: Expr,
        then_: Vec<Node>,
        elifs: Vec<(Expr, Vec<Node>)>,
        else_: Option<Vec<Node>>,
    },
    For {
        item: Pattern,
        iter: Expr,
        key: Option<Expr>,
        body: Vec<Node>,
        empty: Option<Vec<Node>>,
    },
    Await {
        promise: Expr,
        loading: Vec<Node>,
        then_: Option<(Pattern, Vec<Node>)>,
        catch: Option<(Pattern, Vec<Node>)>,
    },
    Slot {
        name: String,
        props: Vec<(String, Expr)>,
        fallback: Vec<Node>,
    },
    SlotFill {
        target: String,
        lets: Vec<(String, String)>,
        body: Vec<Node>,
    },
    /// `<:host attr={expr} aria-foo={cond} />` — declarative reactive
    /// bindings on the component's host element. Replaces the
    /// `mount { el.setAttribute(...) } effect on X { ... }` ritual
    /// every form-associated and ARIA-aware port had to write.
    /// Self-closing only; carries no children, no events.
    Host {
        attrs: Vec<Attr>,
    },
    RawHtml(Expr),
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum AttrValue {
    Static(String),
    Interpolated(Expr),
    BindTwoWay(Expr),                      // bind:value={x}
    OnEvent {
        event: String,
        modifiers: Vec<EventModifier>,
        handler: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct EventModifier {
    pub kind: String,                      // "preventDefault", "stopPropagation",
                                           // "key", "debounce", ...
    pub arg: Option<String>,               // "Enter" for key, "300" for debounce
}

#[derive(Debug, Clone)]
pub struct PropBinding {
    pub name: String,
    pub value: Expr,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct EventBinding {
    pub event: String,
    pub modifiers: Vec<EventModifier>,
    pub handler: Expr,
}

#[derive(Debug, Clone)]
pub struct SlotFill {
    pub target: String,
    pub lets: Vec<(String, String)>,
    pub body: Vec<Node>,
}

/// JS expression — verbatim source string, parsed lazily later if
/// needed (e.g., for dependency analysis). The optional [`Span`] points
/// at the inner expression text (excluding wrapping `{` / `}` for
/// interpolations) so diagnostics anchor at the offending tokens.
#[derive(Debug, Clone)]
pub struct Expr {
    pub raw: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Tuple(Vec<String>),                    // `(item, i)` in `for (item, i) of list`
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub raw: String,
    /// Byte offset of `raw` within the original source. Used by token
    /// reference checking to map a `${…}` body offset back to a
    /// `(line, col)` for the diagnostic.
    pub body_byte_offset: usize,
    pub global: bool,
    pub src: Option<String>,               // <style src="./x.css">
}
