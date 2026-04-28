//! Lowered IR types — what the emitter consumes.

#[derive(Debug, Default)]
pub struct LoweredFile {
    pub static_html: String,
    pub bindings: Vec<RuntimeBinding>,
    pub scoped_css: String,
}

#[derive(Debug)]
pub struct RuntimeBinding {
    pub anchor: BindingAnchor,
    pub kind: BindingKind,
    pub deps: Vec<DepRef>,
    pub evaluator: String,                 // JS expression source
}

#[derive(Debug)]
pub enum BindingAnchor {
    /// `data-bid="bN"` element selector — runtime querySelectors.
    DataBid(String),
    /// Comment placeholder used by `for`/`if` blocks.
    CommentMarker(String),
}

#[derive(Debug)]
pub enum BindingKind {
    Text,
    Show,
    Attr(String),
    Class(String),
    BindValue,
    Event { event: String, modifiers: Vec<String> },
    For { item: String, index: Option<String>, key: Option<String> },
    If { has_else: bool },
}

#[derive(Debug, Clone)]
pub enum DepRef {
    State(String),
    Derived(String),
    Data(String),
    Prop(String),
}
