//! Control-flow rendering for `{#if}`, `{#for}`, `{#match}`.
//!
//! Phase D-3 fills in the bodies of control-flow constructs against the
//! initial scope. When the discriminating expression evaluates inside
//! the renderable subset and produces a usable value, the matching
//! branch / iteration / case is rendered inline. Otherwise we fall back
//! to D-1's empty `<template data-set-{kind}="bN"></template>`
//! placeholder so the attach-only client runtime takes over.
//!
//! `{#await}` is intentionally absent — the IR does not yet model an
//! `Instr::Await` variant, and the D-3 spec defers IR extension to a
//! later phase. `<while-loading>` / `<while-error>` are element flags
//! that pair with `<data>` envelopes, not control-flow instructions.

use serde_json::Value;

use crate::ir::{ForInstr, IfInstr, Instr, MatchInstr};

use super::escape::escape_attr_value;
use super::eval::{eval, strict_eq, stringify, truthy};
use super::walker::Walker;

pub(super) fn render_if(walker: &mut Walker, instr: &IfInstr) {
    let key = walker.keys.alloc();

    for branch in &instr.branches {
        match eval(&branch.cond.source, &walker.scope) {
            Ok(value) => {
                if truthy(&value) {
                    open_template(walker, "if", &key);
                    walker.walk_block(&branch.body);
                    close_template(walker);
                    return;
                }
            }
            Err(_) => {
                empty_template(walker, "if", &key);
                return;
            }
        }
    }

    if let Some(else_body) = &instr.else_body {
        open_template(walker, "if", &key);
        walker.walk_block(else_body);
        close_template(walker);
    } else {
        empty_template(walker, "if", &key);
    }
}

pub(super) fn render_match(walker: &mut Walker, instr: &MatchInstr) {
    let key = walker.keys.alloc();

    let disc = match eval(&instr.discriminant.source, &walker.scope) {
        Ok(v) => v,
        Err(_) => {
            empty_template(walker, "match", &key);
            return;
        }
    };

    for case in &instr.cases {
        match eval(&case.test.source, &walker.scope) {
            Ok(test_val) => {
                if strict_eq(&disc, &test_val) {
                    open_template(walker, "match", &key);
                    walker.walk_block(&case.body);
                    close_template(walker);
                    return;
                }
            }
            Err(_) => {
                empty_template(walker, "match", &key);
                return;
            }
        }
    }

    if let Some(default) = &instr.default {
        open_template(walker, "match", &key);
        walker.walk_block(default);
        close_template(walker);
    } else {
        empty_template(walker, "match", &key);
    }
}

pub(super) fn render_for(walker: &mut Walker, instr: &ForInstr) {
    let key = walker.keys.alloc();

    let value = match eval(&instr.iterable.source, &walker.scope) {
        Ok(v) => v,
        Err(_) => {
            empty_template(walker, "for", &key);
            return;
        }
    };

    let array = match value {
        Value::Array(a) => a,
        _ => {
            empty_template(walker, "for", &key);
            return;
        }
    };

    if array.is_empty() {
        if let Some(else_body) = &instr.else_body {
            open_template(walker, "for", &key);
            walker.walk_block(else_body);
            close_template(walker);
        } else {
            empty_template(walker, "for", &key);
        }
        return;
    }

    open_template(walker, "for", &key);

    let body_start = walker.keys.snapshot();
    let mut body_end = body_start;
    for (idx, item) in array.iter().enumerate() {
        if idx > 0 {
            walker.keys.restore(body_start);
        }

        let prior_var = walker
            .scope
            .insert(instr.var_name.clone(), item.clone());
        let prior_idx = instr
            .index_name
            .as_ref()
            .map(|n| (n.clone(), walker.scope.insert(n.clone(), Value::from(idx as i64))));

        let key_attr_value = instr.key.as_ref().map(|k| {
            eval(&k.source, &walker.scope)
                .map(|v| stringify_key(&v))
                .unwrap_or_default()
        });

        walk_iteration_body(walker, &instr.body, key_attr_value.as_deref());

        body_end = walker.keys.snapshot();

        match prior_var {
            Some(v) => {
                walker.scope.insert(instr.var_name.clone(), v);
            }
            None => {
                walker.scope.remove(&instr.var_name);
            }
        }
        if let Some((name, prior)) = prior_idx {
            match prior {
                Some(v) => {
                    walker.scope.insert(name, v);
                }
                None => {
                    walker.scope.remove(&name);
                }
            }
        }
    }

    walker.keys.restore(body_end);

    close_template(walker);
}

/// Render a `{#for}` iteration body, optionally injecting the
/// `data-set-key` marker. When the body has a single element root, the
/// attribute attaches to that element; otherwise we wrap the iteration
/// in a `<template data-set-key="…">` so reconciliation still has a
/// per-iteration anchor.
fn walk_iteration_body(walker: &mut Walker, body: &[Instr], key_attr_value: Option<&str>) {
    let Some(key_val) = key_attr_value else {
        walker.walk_block(body);
        return;
    };

    let element_positions: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, i)| matches!(i, Instr::Element(_)))
        .map(|(i, _)| i)
        .collect();

    if element_positions.len() == 1 {
        let target = element_positions[0];
        for (i, instr) in body.iter().enumerate() {
            if i == target {
                if let Instr::Element(el) = instr {
                    walker.walk_element_with_extra(el, &[("data-set-key", key_val)]);
                }
            } else {
                walker.walk_instr(instr);
            }
        }
        return;
    }

    walker.push_raw(&format!(
        "<template data-set-key=\"{}\">",
        escape_attr_value(key_val)
    ));
    walker.walk_block(body);
    walker.push_raw("</template>");
}

/// Stringify a value for use as a `data-set-key` attribute. JSON
/// scalars use their evaluator-string form (`42` → `"42"`, `"alice"` →
/// `"alice"`, `true` → `"true"`, `null` → `"null"`); objects/arrays
/// fall back to a stable JSON dump so reconciliation has *something* to
/// key on (the authoring smell is real but the framework allows it).
fn stringify_key(v: &Value) -> String {
    match v {
        Value::Object(_) | Value::Array(_) => {
            serde_json::to_string(v).unwrap_or_default()
        }
        _ => stringify(v),
    }
}

fn open_template(walker: &mut Walker, kind: &str, key: &str) {
    walker.push_raw(&format!("<template data-set-{kind}=\"{key}\">"));
}

fn close_template(walker: &mut Walker) {
    walker.push_raw("</template>");
}

fn empty_template(walker: &mut Walker, kind: &str, key: &str) {
    walker.push_raw(&format!("<template data-set-{kind}=\"{key}\"></template>"));
}
