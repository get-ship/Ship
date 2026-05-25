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

#[test]
fn live_data_emits_subscribe_url_in_island() {
    use crate::parse::parse;
    let src = r#"<data name="overview" source="ship.git.overview()" live /><p>hello</p>"#;
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "git".to_string() };
    let out = render_default(&program, &opts).expect("render");
    // The JSON island must contain the subscribe URL.
    assert!(
        out.init.get("__subscribe__").and_then(|v| v.as_str()) == Some("/_sui/views/git/sse"),
        "expected __subscribe__ = /_sui/views/git/sse in init, got: {:?}",
        out.init
    );
    assert!(out.html.contains("/_sui/views/git/sse"), "HTML island must contain subscribe URL");
}

#[test]
fn non_live_data_omits_subscribe_url() {
    use crate::parse::parse;
    let src = r#"<data name="info" source="ship.git.overview()" /><p>hello</p>"#;
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "git".to_string() };
    let out = render_default(&program, &opts).expect("render");
    assert!(
        out.init.get("__subscribe__").is_none(),
        "non-live data must not emit __subscribe__, got: {:?}",
        out.init
    );
}

#[test]
fn mixed_live_and_non_live_emits_subscribe_url() {
    use crate::parse::parse;
    let src = r#"<data name="a" source="ship.git.overview()" live /><data name="b" source="ship.git.diff()" /><p>x</p>"#;
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "myview".to_string() };
    let out = render_default(&program, &opts).expect("render");
    assert_eq!(
        out.init.get("__subscribe__").and_then(|v| v.as_str()),
        Some("/_sui/views/myview/sse"),
    );
}
