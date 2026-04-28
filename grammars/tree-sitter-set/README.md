# tree-sitter-set

Tree-sitter grammar for the Sail `.set` single-file component format.

## Status

V0 scaffold. Parses counter-style fixtures cleanly. Iterative work
(per-fixture validation against the 18 production ports in
`packages/ship-set-sail/components/`) is in flight.

## Architecture

Thin outer grammar identifies the three top-level blocks
(`<script>`, `<template>`, `<style>`) and the framework-specific
declarations inside `<script>` (prop / state / derived / data / effect /
event / mount / unmount / function / component / `on form_*`).

Anything that isn't a recognized framework keyword is captured as an
opaque `js_chunk` (script) or as opaque `_brace_body` (template). A
follow-up will inject existing tree-sitter grammars
(`tree-sitter-typescript`, `tree-sitter-html`, `tree-sitter-css`) for
those chunks via `queries/injections.scm` so we don't re-implement the
host languages.

## Build

```bash
npm install
npx tree-sitter generate
npx tree-sitter test
```

## Validation corpus

The 18 `.set` files in `packages/ship-set-sail/components/` are the
real corpus. The grammar is "done" when:

```bash
for f in packages/ship-set-sail/components/*.set; do
  npx tree-sitter parse "$f" --quiet || echo "FAIL: $f"
done
```

prints nothing.
