//! Unit tests for the render module's helpers.

use super::*;

#[test]
fn marker_attr_name_covers_dynamic_kinds() {
    use crate::diagnostics::Span;
    use crate::ir::Expr;

    let span = Span::new(0, 0);
    let expr = || Expr::new("x", span, false);

    assert_eq!(
        marker_attr_name(&AttrInstr::Static {
            name: "class".into(),
            value: "x".into(),
            span,
        }),
        None
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::Expr {
            name: "href".into(),
            expr: expr(),
            span,
        })
        .as_deref(),
        Some("attr:href")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::BindValue { expr: expr(), span }).as_deref(),
        Some("bind:value")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::BindChecked { expr: expr(), span }).as_deref(),
        Some("bind:checked")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::On {
            event: "click".into(),
            handler: expr(),
            modifiers: vec![],
            span,
        })
        .as_deref(),
        Some("on:click")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::Class {
            name: "active".into(),
            expr: expr(),
            span,
        })
        .as_deref(),
        Some("class:active")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::Style {
            name: "--c".into(),
            expr: expr(),
            span,
        })
        .as_deref(),
        Some("style:--c")
    );
    assert_eq!(
        marker_attr_name(&AttrInstr::DynamicAttr {
            name_expr: expr(),
            expr: expr(),
            span,
        })
        .as_deref(),
        Some("attr-dynamic")
    );
}

#[test]
fn control_flow_kind_classifies_correctly() {
    use crate::diagnostics::Span;
    use crate::ir::{Expr, ForInstr, IfBranch, IfInstr, MatchInstr};
    let span = Span::new(0, 0);
    let expr = Expr::new("x", span, false);

    let if_instr = Instr::If(IfInstr {
        branches: vec![IfBranch { cond: expr.clone(), body: vec![] }],
        else_body: None,
        span,
    });
    let for_instr = Instr::For(ForInstr {
        var_name: "x".into(),
        index_name: None,
        iterable: expr.clone(),
        key: None,
        body: vec![],
        else_body: None,
        span,
    });
    let match_instr = Instr::Match(MatchInstr {
        discriminant: expr.clone(),
        cases: vec![],
        default: None,
        span,
    });
    assert_eq!(control_flow_kind(&if_instr), Some("if"));
    assert_eq!(control_flow_kind(&for_instr), Some("for"));
    assert_eq!(control_flow_kind(&match_instr), Some("match"));
    assert_eq!(
        control_flow_kind(&Instr::StaticText { text: "x".into(), span }),
        None
    );
}
