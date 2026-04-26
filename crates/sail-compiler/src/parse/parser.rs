//! Parser. Walks the source and emits an AST.
//!
//! See `parse::parse` (the public entry) and the AST in `ast.rs`.
//!
//! Implementation phasing per `02-compiler-architecture.md`:
//! 1. Three-block extraction (script, template, style) — easiest first
//! 2. Script-block keyword scanner (props, state, derived, data, ...)
//! 3. Template parser — HTML elements + interpolations + control flow
//! 4. Style attributes (`global`, `src`)
