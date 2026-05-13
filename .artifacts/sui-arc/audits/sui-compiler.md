# Audit — `crates/sui-compiler` (Rust)

## Inventory

- 79 source files, 15,532 LOC
- Modules: `parse`, `lower`, `check`, `render`, `emit`, `bindings`, `keys`, `islands`, `diagnostics`, `ir`
- 21 test suites, 318 tests pre-audit
- 1 fuzz target pre-audit (`parse_fragment`)

## Bugs found

Six panic bugs discovered by `compile_full` and `bindings_build` fuzz
runs. Same general class — byte-indexed slicing of `&str` without
char-boundary handling. Fixed:

| # | Site | Trigger | Commit |
|---|---|---|---|
| 1 | `render/eval/lex.rs:55,66` | Multi-byte char after HTML entity decode (e.g. `°` from `&deg;`) — punct-peek slice mid-char | `862f9b94` |
| 2 | `render/eval/lex.rs:138,171` | Multi-byte char immediately after `\` escape in string/template literal | `0d29f587` |
| 3 | `check/expr_subset.rs:195,213` | Trailing `\` overshot `bytes.len()` in `skip_string` / `skip_template_literal` | `0d29f587` |
| 4 | `emit/expr_rewrite.rs:86` | Template-literal recursion sliced with byte indices that fell inside a multi-byte char | `d5797cff` |
| 5 | `parse/template/blocks/for_header.rs:114` | `{#for ... key=}` scan compared a byte slice over multi-byte chars in the iter binding | `f34c7ca8` |
| 6 | `check/expr_subset.rs:242` | Unterminated `${`-body left `k > bytes.len()`, recursive return propagated overshoot | `d3470777` |

Five regression tests pinned at `tests/fuzz_regression.rs` so each
input fails loud without needing a fuzz session.

## Fuzz coverage expanded

| Target | LOC | What it covers | Status |
|---|---|---|---|
| `parse_fragment` | 29 | HTML parser only (pre-existing) | clean 832k+ iterations |
| `compile_full` (new) | 50 | Full pipeline: parse → check → bindings → lower → emit. View + Component mode | clean 407k+ iterations (after fixes) |
| `bindings_build` (new) | 24 | parse → bindings::build (binding-map IR walk) | clean 745k+ iterations |

Total ~2M iterations across targets, zero panics post-fix. Recommended
CI integration: run each fuzz target for 60s as a smoke gate.

## Follow-ups

- F-C2 (`.artifacts/sui-arc/follow-ups.md`): `expr_rewrite` byte-casts to char,
  silently mangling multi-byte chars in template-literal string contents.
  Fixed the panic class around it; the correctness gap is logged for a
  half-day rewrite using `char_indices()`.

## Decisions surfaced

None requiring user input — every bug had an unambiguous fix.
