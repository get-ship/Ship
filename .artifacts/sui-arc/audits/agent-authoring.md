# Audit — Agent authoring story for SUI

(Verbatim from the audit pass. Acting on highest-leverage items in the
arc commits below; the rest are tracked here for follow-up sessions.)

## Critical gaps

1. **No MCP surface for SUI authoring.** Agents can't `sui.compile(path)`
   to validate `.sui` without an open editor LSP. Closes the loop for
   any agent working over the file system.
2. **`<data source="ship.X.Y()">` is not type-checked.** Wrong SDK calls
   fail at runtime, not compile time. Agent has no early signal.
3. **No `<data> sources` skill doc.** No catalog of available SDK calls
   from the agent's perspective.

## High-impact lints missing

- `missing-for-key` — `{#for}` without `key={…}` is a common silent bug
  on list reorder. **Acted on this session.**
- `event-name-validation` — typos like `on:Click` accept silently.
  **Acted on this session.**
- Compiler + LSP rules duplicated. Should converge on `crates/sui-lint`
  as single source of truth.

## LSP gaps

- No snippets for `{#for}` / `{#if}` / `{#match}` blocks
- No CEM-driven prop autocomplete for `<sui-*>` (Phase 6 TODO)
- No code actions / quick-fixes
- No go-to-definition / rename
- Test coverage: 12 files / 65+ tests but no error-recovery coverage

## Compiler diagnostics quality

Sampled 7 known-broken cases. Spans good. Messages mixed — some have
`suggest:` text, others just "requires X". Examples that are actionable:

- `"<state> is no longer supported; declare reactive state in the
  controller's scope: factory"` — good, names replacement.
- `"arrow functions are only allowed in on: event handlers"` — good.

Examples that fall short:

- `on:Click={…}` (typo) — **silent**, no diagnostic at all.
- `<data source="ship.bogus()" />` — silent until runtime.
- Some "requires X" messages don't say "add Y" — `bind:this requires a
  bare identifier` says what's wrong but not what the user should do.

## Skill docs

Inventory of `.ship/skills/ship-views/`:
- `SKILL.md` — current (2026-05-05), accurate against modern controller
  shape, file layout section explains both new + legacy.
- `references/template-surface.md` — covers control flow, bindings,
  slots, hooks. Solid.
- `references/sui-internals.md` — for framework authors. Solid.

Cross-check of "Hard rules":
- ✓ 6 of 8 enforced by lint/compiler/parser
- ✗ 2 of 8 enforced only by author discipline (no `customElements.define`;
  subscriptions only in `<data>`)

Big skill-doc gaps:
- No `<data>` skill (data sources, signatures, examples)
- No `<prop>` deep dive (validation, reflection, enum values)
- No troubleshooting guide for common diagnostics
- No "build a form view" / "add data subscription" pattern guide

## Action items for THIS session

1. **`missing-for-key` lint rule** — acted on
2. **`event-name-validation` lint rule** — acted on
3. **Port primitives from Lit** — sequenced after lints

## Action items for follow-up sessions

| Priority | Item | Estimate |
|---|---|---|
| Critical | MCP `sui.compile()` / `sui.lint()` tool | ~150 LOC Rust + manifest |
| Critical | `<data source=` type validation | ~300 LOC + SDK schema |
| Critical | Write `<data> sources` skill | ~500 words |
| High | Code actions in LSP (add `key=`, etc.) | ~150 LOC |
| High | LSP snippets for control flow | ~100 LOC |
| High | Compiler + LSP rule unification | Refactor |
| Medium | CEM-driven prop autocomplete | Phase 6 TODO |
| Medium | Troubleshooting skill section | ~200 words |
| Low | `<prop>` deep dive skill | ~400 words |
