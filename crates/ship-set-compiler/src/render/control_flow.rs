//! Control-flow rendering for `{#if}` and `{#match}`. `{#for}` lives
//! in `render_for.rs` (large enough body to push this file past the
//! 300-line cap).
//!
//! Phase D-3 fills in the bodies of control-flow constructs against the
//! initial scope. When the discriminating expression evaluates inside
//! the renderable subset and produces a usable value, the matching
//! branch / case is rendered inline. Otherwise we fall back to D-1's
//! empty `<template data-set-{kind}="bN"></template>` placeholder so
//! the attach-only client runtime takes over.
//!
//! `{#await}` is intentionally absent — the IR does not yet model an
//! `Instr::Await` variant.
//!
//! ## Phase E-prep — registry parity
//!
//! The walker's `b<N>` keys must match the registry index-for-index.
//! The registry pre-pass walks every branch (it has no SSR scope) and
//! allocates one key per source-position-bearing-a-binding. To stay
//! aligned, the walker also walks every branch — but suppresses HTML
//! output for the un-rendered ones via `walk_block_silent`.
//!
//! ## Phase E-2 — fragment mode
//!
//! When the walker is rendering a fragment for a specific boundary key,
//! `walker.enter_target_if_match(&key)` returns true at the matching
//! `{#if}` / `{#for}` / `{#match}`. In that case the wrapper (active
//! `<div>` or empty `<template>`) is suppressed: the runtime already
//! has the wrapper in place and is swapping the wrapper's children, not
//! the wrapper itself. The body is emitted with output forced visible
//! by `enter_target_if_match`; `exit_target` restores silent mode so
//! the rest of the IR walks without producing more HTML.

use crate::ir::{IfInstr, MatchInstr};

use super::eval::{eval, strict_eq, truthy};
use super::render_for;
use super::template_helpers::{close_active, empty_template, open_active};
use super::walker::Walker;

pub(super) use render_for::render_for;

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

    let is_target = walker.enter_target_if_match(&key);

    if error {
        walk_all_silent(walker, &instr.branches, instr.else_body.as_deref());
        if !is_target {
            empty_template(walker, "if", &key);
        }
    } else if let Some(t) = taken {
        for (i, branch) in instr.branches.iter().enumerate() {
            if i == t {
                if !is_target {
                    open_active(walker, "if", &key);
                }
                walker.walk_block(&branch.body);
                if !is_target {
                    close_active(walker);
                }
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
            if !is_target {
                open_active(walker, "if", &key);
            }
            walker.walk_block(else_body);
            if !is_target {
                close_active(walker);
            }
        } else if !is_target {
            empty_template(walker, "if", &key);
        }
    }

    if is_target {
        walker.exit_target();
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

    let is_target = walker.enter_target_if_match(&key);

    if error {
        for case in &instr.cases {
            walker.walk_block_silent(&case.body);
        }
        if let Some(default) = &instr.default {
            walker.walk_block_silent(default);
        }
        if !is_target {
            empty_template(walker, "match", &key);
        }
    } else if let Some(t) = taken {
        for (i, case) in instr.cases.iter().enumerate() {
            if i == t {
                if !is_target {
                    open_active(walker, "match", &key);
                }
                walker.walk_block(&case.body);
                if !is_target {
                    close_active(walker);
                }
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
            if !is_target {
                open_active(walker, "match", &key);
            }
            walker.walk_block(default);
            if !is_target {
                close_active(walker);
            }
        } else if !is_target {
            empty_template(walker, "match", &key);
        }
    }

    if is_target {
        walker.exit_target();
    }
}
