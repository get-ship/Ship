//! Control-flow rendering for `{#if}`, `{#match}`, `{#while-loading}`,
//! `{#while-error}`. `{#for}` lives in `render_for.rs`.
//!
//! Phase D-3 fills in the bodies of control-flow constructs against the
//! initial scope. When the discriminating expression evaluates inside
//! the renderable subset and produces a usable value, the matching
//! branch / case is rendered inline. Otherwise we fall back to D-1's
//! `<sui-{kind} key="bN" hidden></sui-{kind}>` placeholder so
//! the attach-only client runtime takes over.
//!
//! Phase 2 deletes the `/_sui/render-fragment/` POST endpoint, so the
//! `enter_target_if_match` / `exit_target` plumbing is gone — the
//! walker only ever produces full first-paint HTML now. Client-side
//! re-render kicks in via the compiler-emitted `__b<N>_attach`
//! handlers.
//!
//! ## Phase E-prep — registry parity
//!
//! The walker's `b<N>` keys must match the registry index-for-index.
//! For `{#while-loading}` / `{#while-error}`, the registry pre-pass walks
//! the body under suppression (allocates keys but marks them suppressed).
//! The render walker must also advance its counter by the same number of
//! keys. We call `walk_block_silent` for the non-active case and
//! `walk_block` for the active case (which emits the HTML body and also
//! advances the counter).

use crate::ir::{IfInstr, MatchInstr, WhileErrorInstr, WhileLoadingInstr};

use super::eval::{eval, strict_eq, truthy};
use super::render_for;
use super::template_helpers::{close_active, empty_template, open_active, open_active_with_branch};
use super::walker::Walker;

pub(super) use render_for::render_for;

/// Evaluate whether a signal name refers to a loading-state envelope.
fn is_loading(bind: &str, walker: &Walker<'_>) -> bool {
    use super::eval::eval;
    match eval(&format!("{bind}.kind"), &walker.scope) {
        Ok(v) => matches!(v, serde_json::Value::String(s) if s == "loading"),
        Err(_) => true, // default: assume loading when unevaluable
    }
}

/// Evaluate whether a signal name is in error state.
fn is_error(bind: &str, walker: &Walker<'_>) -> bool {
    use super::eval::eval;
    match eval(&format!("{bind}.kind"), &walker.scope) {
        Ok(v) => matches!(v, serde_json::Value::String(s) if s == "error"),
        Err(_) => false,
    }
}

pub(super) fn render_while_loading(walker: &mut Walker<'_>, instr: &WhileLoadingInstr) {
    let key = walker.next_key();
    let active = is_loading(&instr.bind, walker);

    if active {
        open_active(walker, "while-loading", &key);
        walker.walk_block(&instr.body);
        close_active(walker, "while-loading");
    } else {
        walker.walk_block_silent(&instr.body);
        empty_template(walker, "while-loading", &key);
    }
}

pub(super) fn render_while_error(walker: &mut Walker<'_>, instr: &WhileErrorInstr) {
    let key = walker.next_key();
    let active = is_error(&instr.bind, walker);

    if active {
        open_active(walker, "while-error", &key);
        walker.walk_block(&instr.body);
        close_active(walker, "while-error");
    } else {
        walker.walk_block_silent(&instr.body);
        empty_template(walker, "while-error", &key);
    }
}

pub(super) fn render_if(walker: &mut Walker<'_>, instr: &IfInstr) {
    let key = walker.next_key();

    let mut taken: Option<usize> = None;
    let mut error = false;
    for (i, branch) in instr.branches.iter().enumerate() {
        match eval(&branch.cond.source, &walker.scope) {
            Ok(value) => {
                if truthy(&value) {
                    taken = Some(i);
                    break;
                }
            }
            Err(_) => {
                error = true;
                break;
            }
        }
    }

    if error {
        walk_all_silent(walker, &instr.branches, instr.else_body.as_deref());
        empty_template(walker, "if", &key);
    } else if let Some(t) = taken {
        for (i, branch) in instr.branches.iter().enumerate() {
            if i == t {
                open_active(walker, "if", &key);
                walker.walk_block(&branch.body);
                close_active(walker, "if");
            } else {
                walker.walk_block_silent(&branch.body);
            }
        }
        if let Some(else_body) = &instr.else_body {
            walker.walk_block_silent(else_body);
        }
    } else {
        for branch in &instr.branches {
            walker.walk_block_silent(&branch.body);
        }
        if let Some(else_body) = &instr.else_body {
            open_active(walker, "if", &key);
            walker.walk_block(else_body);
            close_active(walker, "if");
        } else {
            empty_template(walker, "if", &key);
        }
    }
}

fn walk_all_silent(
    walker: &mut Walker<'_>,
    branches: &[crate::ir::IfBranch],
    else_body: Option<&[crate::ir::Instr]>,
) {
    for branch in branches {
        walker.walk_block_silent(&branch.body);
    }
    if let Some(else_body) = else_body {
        walker.walk_block_silent(else_body);
    }
}

pub(super) fn render_match(walker: &mut Walker<'_>, instr: &MatchInstr) {
    let key = walker.next_key();
    let disc_result = eval(&instr.discriminant.source, &walker.scope);
    let mut taken: Option<usize> = None;
    let mut error = disc_result.is_err();

    if let Ok(disc) = &disc_result {
        for (i, case) in instr.cases.iter().enumerate() {
            match eval(&case.test.source, &walker.scope) {
                Ok(test_val) => {
                    if strict_eq(disc, &test_val) {
                        taken = Some(i);
                        break;
                    }
                }
                Err(_) => {
                    error = true;
                    break;
                }
            }
        }
    }

    if error {
        for case in &instr.cases {
            walker.walk_block_silent(&case.body);
        }
        if let Some(default) = &instr.default {
            walker.walk_block_silent(default);
        }
        empty_template(walker, "match", &key);
    } else if let Some(t) = taken {
        for (i, case) in instr.cases.iter().enumerate() {
            if i == t {
                open_active_with_branch(walker, "match", &key, &format!("when_{i}"));
                walker.walk_block(&case.body);
                close_active(walker, "match");
            } else {
                walker.walk_block_silent(&case.body);
            }
        }
        if let Some(default) = &instr.default {
            walker.walk_block_silent(default);
        }
    } else {
        for case in &instr.cases {
            walker.walk_block_silent(&case.body);
        }
        if let Some(default) = &instr.default {
            open_active_with_branch(walker, "match", &key, "when_default");
            walker.walk_block(default);
            close_active(walker, "match");
        } else {
            empty_template(walker, "match", &key);
        }
    }
}
