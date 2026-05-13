# Audit — `packages/sui-dom`

## Inventory

- 31 source files, 3,142 LOC
- Modules: `hydrate/` (12 files), `template/` (8 files), `legacy/` (5 files), `bindings/` (1 file), top-level (`index`, `morph`, `preprocess`, `setup-loader`)
- Test files: `bindings.test.ts`, `controller-hydrate.test.ts`, `directives.test.ts`, `git-view.test.ts`, `hydrate.test.ts`, `parse.test.ts`, `reactive-data.test.ts`, `template-hooks.test.ts`, `template.test.ts`, plus `hydrate/` subdir with per-binding tests

## Pre-audit state

Test suite was broken: 13 test files failed to load due to stale
`@ship/ui-runtime` aliases in `vitest.config.ts` left over from the
TS package rename. Fixed in 15c2cfc4 by sweeping `../ui-runtime/` →
`../sui-runtime/` etc. Suite went from 0 runnable → 79/79.

## Source walk findings

- `src/index.ts` — re-exports hydrate primitives + `setup-loader`. Legacy
  parser intentionally excluded; comment documents the 5KB-gzip cost of
  pulling it in.
- `src/hydrate/walk.ts` — clean. Throws a helpful diagnostic with
  parent tag + child count when the compiler-emitted path falls off.
  21 LOC, no smells.
- `src/morph.ts` — minimal DOM morph (147 LOC). Documented limitations:
  no `onBeforeNodeDiscarded` hook, no SVG namespace special-case, no
  file-input value preservation. All explicit in module comment.
- `src/template/{compile,tokenize,render*,expr,compile-blocks}.ts` —
  1,277 LOC across 8 files. Zero TODO/FIXME/HACK in tree. Clean
  separation: tokenize → compile → render.
- `src/bindings/attr-compile.ts` — clean. Defensive throws on malformed
  attribute names + missing braces. All recognized prefixes covered.
- `src/legacy/` — parse-on-client path retained for tests and the
  `@ship/sui-eval` legacy fixture-runner. Module comment clear that
  importing legacy from a production view pulls ~5KB gzip.

No bugs found in the source walk. The audit was looking for correctness
issues; what surfaced was missing edge-case test coverage on `morph`
(see Tests added below).

## Tests added

`test/morph.test.ts` — 14 new tests covering:

- Attribute update / removal / identity preservation
- Wholesale replace on tag mismatch
- Keyed reorder via `id` + `data-key`
- Removal of dropped keys + insertion of new keys
- Position-match unkeyed siblings
- Text node morph
- Documented limitations (multi-root drop, whitespace no-op, comment-only no-op)
- Focus preservation across same-tag morph

All 14 pass against current implementation — locks in documented
behavior including the limitations.

## Test totals

- Before audit: 0 runnable (workspace alias bug)
- After alias fix: 79 passed
- After morph additions: 93 passed

## Follow-ups

None surfaced. The legacy path's "delete at SUI parity" status remains
the long-term cleanup target; not in scope for this audit.

## Decisions surfaced

None requiring user input.
