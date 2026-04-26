//! Phase 1 — parser. Turns `.sail` source into an AST.
//!
//! Responsibilities:
//! - Three-block extraction: `<script>`, `<template>`, `<style>`
//! - Block attributes (`<script lang="ts">`, `<style src="./x.css">`)
//! - Template parsing: HTML elements, `{expr}` interpolations,
//!   `{#if}`/`{#for}`/`{#await}` blocks, `bind:`/`on:`/`slot=` attributes
//! - Script parsing: top-level keyword declarations
//!   (`prop`, `state`, `derived`, `data`, `effect`, `event`, `function`)
//! - Style parsing: minimal CSS tokenizer
//!
//! See `.artifacts/specs/sail/01-language-spec.md` for the language
//! and `.artifacts/specs/sail/02-compiler-architecture.md` for phase 1
//! implementation guidance.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod script;

pub use ast::*;

use anyhow::Result;

/// Parse a `.sail` source string into a `SailFile` AST.
///
/// Phase 1 in stages:
/// - Three-block extraction (implemented; `parser::extract_blocks`)
/// - Block attributes (script lang, style global+src) — implemented
/// - Script body keyword scan — landing next per spec
/// - Template body parsing (HTML + interpolations + control flow) — landing next
pub fn parse(source: &str) -> Result<SailFile> {
    parser::parse_file(source)
}
