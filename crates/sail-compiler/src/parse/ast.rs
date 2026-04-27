//! AST types for the Sail parser.
//!
//! Mirrors the language spec at `.artifacts/specs/sail/01-language-spec.md`.

#[derive(Debug, Clone, Default)]
pub struct SailFile {
    pub script: Option<ScriptBlock>,
    pub template: Option<Template>,
    pub style: Option<StyleBlock>,
    pub source_map: SourceMap,
}

#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// Original source for span resolution.
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ScriptBlock {
    pub lang: ScriptLang,
    pub raw: String,                       // verbatim, for type-stripping
    pub props: Vec<PropDecl>,
    pub state: Vec<StateDecl>,
    pub derived: Vec<DerivedDecl>,
    pub data: Vec<DataDecl>,
    pub effects: Vec<EffectDecl>,
    pub events: Vec<EventDecl>,
    pub mount: Option<String>,
    pub unmount: Option<String>,
    pub functions: Vec<FunctionDecl>,
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
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    pub ty: Option<String>,
    pub initial: String,                   // verbatim JS
}

#[derive(Debug, Clone)]
pub struct DerivedDecl {
    pub name: String,
    pub expr: String,
}

#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    pub ty: Option<String>,
    pub expr: String,                      // `ship.x.y(args)` etc.
    pub live: bool,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub deps: Vec<String>,                 // `effect on a, b { ... }` → ["a","b"]
    pub once: bool,                        // `effect once { ... }`
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct EventDecl {
    pub name: String,
    pub payload_ty: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub raw: String,
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
    },
    /// Sail or third-party custom element. Distinguished from Element
    /// by the registry; parser doesn't know yet — analyzer resolves.
    Component {
        tag: String,
        props: Vec<PropBinding>,
        events: Vec<EventBinding>,
        slots: Vec<SlotFill>,
        children: Vec<Node>,
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
    /// Compile-time block placeholder in a layout/parent template.
    /// `<block name="content">default</block>` — the body acts as a
    /// fallback when no `<fill>` from the extending child overrides it.
    Block {
        name: String,
        default: Vec<Node>,
    },
    /// Compile-time block override in a child template that
    /// `<extends>` a parent. Replaces the matching `<block name="X">`
    /// in the parent at resolve time. Always at the top level of the
    /// child's template.
    Fill {
        name: String,
        body: Vec<Node>,
    },
    /// `<include src="./_partial.set" />` — drop-in compile-time
    /// fragment with no parameters. Replaced at resolve time with the
    /// parsed contents of the included file.
    Include {
        src: String,
    },
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
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
/// needed (e.g., for dependency analysis).
#[derive(Debug, Clone)]
pub struct Expr {
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Tuple(Vec<String>),                    // `(item, i)` in `for (item, i) of list`
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub raw: String,
    pub global: bool,
    pub src: Option<String>,               // <style src="./x.css">
}
