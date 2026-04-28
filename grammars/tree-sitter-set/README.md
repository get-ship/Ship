# tree-sitter-set

Tree-sitter grammar for the Sail `.set` single-file component format.

## Status

V1. Every `.set` file in `packages/ship-set-sail/components/` (41
production ports) parses without ERROR nodes. ABI 15. Corpus tests
cover the script declaration set, host/element/attribute forms,
template control flow (`{#if}` / `{#for}` / `{:else}`), HTML comments,
and the external `<style src=…>` shape.

## Architecture

Thin outer grammar identifies the three top-level blocks
(`<script>`, `<template>`, `<style>`) and the framework-specific
declarations inside `<script>` (prop / state / derived / data / effect
/ event / mount / unmount / function / component / `on form_*`).

Right-hand-side expressions, function/effect/mount bodies, attribute
brace-expressions, and CSS bodies are captured as opaque
balanced-delimiter sequences. A follow-up will inject existing
tree-sitter grammars (`tree-sitter-typescript`, `tree-sitter-html`,
`tree-sitter-css`) for those ranges via `queries/injections.scm` so we
don't re-implement the host languages.

The lexer relies on a few precedence carve-outs to keep the host
language and the framework keywords disambiguated at statement
boundaries:

- Identifier-shaped tokens go through tree-sitter's keyword extraction
  so `state`, `derived`, … terminate an expression at a statement
  boundary while still parsing as identifiers inside a function body.
- A dedicated `_member_access` token (`.foo`) keeps
  `ship.dashboard.state()` from misreading `.state` as the keyword.
- Line/block comments are bumped above the balanced-atom token so a
  `//` run that doesn't include any delimiter can't be eaten by an
  atom and (later, via stray apostrophes/backticks) open a phantom
  string or template literal.

## Build

```bash
npm install
npx tree-sitter generate
npx tree-sitter test
```

## Validation corpus

```bash
cd grammars/tree-sitter-set
for f in ../../packages/ship-set-sail/components/*.set; do
  npx tree-sitter parse "$f" --quiet || echo "FAIL: $f"
done
```

Expected: prints nothing.

## Known v2 work

- **External scanner.** Multi-line expressions and CSS bodies currently
  rely on a balanced-rule + precedence approximation. An external C
  scanner would (a) let CSS bodies span literal `<` characters
  (currently we rely on `style_close` winning by length), (b) handle
  template literals containing `${\`}` (a backtick inside `${…}`
  inside a backtick — none in the corpus today), and (c) make the
  expression-vs-keyword disambiguation explicit instead of relying on
  the lexer's keyword-extraction priority.
- **Host-language injections.** `queries/injections.scm` is not yet
  shipped; the long-form opaque ranges (script chunks, template brace
  bodies, style bodies) will pick up `tree-sitter-typescript` /
  `tree-sitter-html` / `tree-sitter-css` once injections land.
- **Locals.** `queries/locals.scm` would expose declaration scopes
  (props, state, derived, function params) for jump-to-def in editors
  that consume the `@local.*` family. Currently absent.
- **HTML attribute namespaces.** `attr_name` excludes `:` so the
  `bind:` / `on:` directives win the lexer race; the corpus has no
  SVG-style namespaced attributes (`xlink:href`, …) so this is safe
  today but worth revisiting if SVG lands as a first-class template
  surface.
