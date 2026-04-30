//! Shared `<template data-set-…>` helpers used by every control-flow
//! renderer.

use super::walker::Walker;

pub(super) fn open_template(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!("<template data-set-{kind}=\"{key}\">"));
}

pub(super) fn close_template(walker: &mut Walker<'_>) {
    walker.push_raw("</template>");
}

pub(super) fn empty_template(walker: &mut Walker<'_>, kind: &str, key: &str) {
    walker.push_raw(&format!("<template data-set-{kind}=\"{key}\"></template>"));
}
