//! Directive declarations — `<data>`, `<state>`, `<component>`, `<prop>`.
//!
//! These are lifted out of the template tree by the parser. The
//! emitter expands each one into the matching runtime call.

use crate::diagnostics::Span;
use crate::ir::expr::Expr;

/// `<data name="agents" source="ship.agents.summaries()" live />`
#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    /// Source expression — read at hydrate time, parameters are
    /// `(ship, route)`. Not braces-stripped; the body is a plain JS
    /// expression literal lifted from the `source=` attribute.
    pub source: Expr,
    pub live: bool,
    pub span: Span,
}

/// `<state name="count" value="0" />` or `value={0}`.
#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    /// Initial-value expression. Brace form (`{...}`) is stripped at
    /// parse time so this is always a bare expression.
    pub value: Expr,
    pub span: Span,
}

/// One `<prop>` inside `<component>`.
#[derive(Debug, Clone)]
pub struct PropDecl {
    pub name: String,
    pub kind: PropKind,
    /// String form of the `default=` attribute; the emitter coerces
    /// this to the right JS literal based on `kind`.
    pub default: Option<String>,
    pub reflect: bool,
    pub bindable: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropKind {
    String,
    Bool,
    Number,
    /// `values="default|primary"` — kept in source order.
    Union(Vec<String>),
}

/// `<component name="ship-button">` declaration.
#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub tag: String,
    pub props: Vec<PropDecl>,
    pub span: Span,
}
