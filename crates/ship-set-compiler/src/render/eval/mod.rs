//! SSR expression evaluator — parses an expression from the renderable
//! subset and executes it against an initial scope.
//!
//! Anything outside the supported subset (arrow callbacks, identifier
//! calls with arguments, dynamic method names, …) returns
//! `EvalError::NonEvaluable` and the walker falls back to D-1's
//! marker-only output.

use std::collections::HashMap;

mod ast;
mod coerce;
mod exec;
mod lex;
mod methods;
mod parser;

#[cfg(test)]
mod tests;

pub use coerce::{truthy, to_string as stringify};
pub use parser::parse_expr;

/// Map of identifier name -> JSON value. Built once per render from
/// the program's `<state>` / `<data>` declarations plus the synthetic
/// `route` placeholder.
pub type Scope = HashMap<String, serde_json::Value>;

/// Failure modes for the evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// Identifier was not in scope.
    Unbound(String),
    /// Operand had the wrong shape (e.g. property access on `null`).
    TypeMismatch(String),
    /// Outside the supported subset (e.g. a `.map` with a non-evaluable
    /// callback). Caller should fall back to marker-only output.
    NonEvaluable(String),
    /// Could not tokenize / parse the expression source.
    ParseError(String),
}

/// Parse and evaluate an expression source against the given scope.
pub fn eval(src: &str, scope: &Scope) -> Result<serde_json::Value, EvalError> {
    let node = parse_expr(src)?;
    exec::execute(&node, scope)
}
