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
    Window {
        events: Vec<EventBinding>,
    },
    Document {
        events: Vec<EventBinding>,
    },
    RawHtml(Expr),
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
