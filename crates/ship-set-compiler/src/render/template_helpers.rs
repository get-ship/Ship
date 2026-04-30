//! Shared boundary-marker helpers used by every control-flow renderer.
//!
//! Two shapes:
//!
//! - **Active branch (rendered, visible)** — `<div data-set-{kind}="bN"
//!   style="display:contents">…body…</div>`. The `display:contents`
//!   wrapper transparently passes its children through to layout, so the
//!   author's HTML renders exactly as if the wrapper weren't there. The
//!   `data-set-{kind}` attribute is the boundary marker the attach
//!   runtime walks.
//! - **Placeholder (un-rendered, invisible)** — `<template
//!   data-set-{kind}="bN"></template>`. The `<template>` element parks
//!   its (empty) content in `template.content`, so nothing renders. When
//!   the runtime decides this branch should activate, it swaps the
//!   `<template>` for a populated `<div>`.
//!
//! Why not always use `<template>`: per the HTML spec, content inside
//! `<template>` lives in a DocumentFragment (`template.content`) and is
//! not part of the live DOM — the browser does not render it. Server-
//! rendered active branches inside `<template>` would be invisible on
//! first paint, which defeats the SSR contract. The active wrapper has
//! to be a regular element; `display:contents` keeps it transparent to
//! layout.

use super::walker::Walker;

pub(super) fn open_active(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!(
        "<div data-set-{kind}=\"{key}\" style=\"display:contents\">"
    ));
}

pub(super) fn close_active(walker: &mut Walker<'_>) {
    walker.push_raw("</div>");
}

pub(super) fn empty_template(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!("<template data-set-{kind}=\"{key}\"></template>"));
}

pub(super) fn open_for_item(walker: &mut Walker<'_>, key_value: &str) {
    walker.push_raw(&format!(
        "<div data-set-key=\"{key_value}\" style=\"display:contents\">"
    ));
}

pub(super) fn close_for_item(walker: &mut Walker<'_>) {
    walker.push_raw("</div>");
}
