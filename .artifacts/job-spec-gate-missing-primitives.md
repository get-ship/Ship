# Job Spec: Gate the missing-primitives branch

**Branch under review:** `job/missing-primitives`
**Worktree on disk:** `/home/madvib/dev/ship-worktrees/missing-primitives/`
**Original work spec:** `.artifacts/job-spec.md` inside that worktree
   (a copy of `.artifacts/job-spec-missing-primitives.md` from the
   commander worktree).
**Agent:** gatekeeper
**Mode:** autonomous

## Goal

Independent verification of the ship-designer's work building five
new SUI primitives — `sui-tag-input`, `sui-combobox`, `sui-autocomplete`,
`sui-date-picker`, `sui-color-picker`. Pass / fail / "almost there"
verdict with concrete evidence the human (or commander) can act on.

The designer agent may still be running when you start. Inspect the
worktree state as it is — uncommitted work counts as work — and report
what passed, what failed, what was skipped or partial.

## File scope (read-only inspection)

`/home/madvib/dev/ship-worktrees/missing-primitives/` — the full
worktree. Do not write files outside `.artifacts/web-qa/` or the gate
report path.

## What to verify

Walk through every acceptance criterion from the original spec.

1. **All five primitive directories exist** at
   `packages/components/primitives/src/{sui-tag-input,sui-combobox,sui-autocomplete,sui-date-picker,sui-color-picker}/`
   with at minimum a `<name>.sui` and (where stateful logic is required)
   a `<name>.controller.ts`.

2. **All five registered in `packages/components/primitives/src/index.ts`**
   (alphabetical insertion).

3. **Compiler enforcement clean for each new primitive.** Run from the
   worktree:
   ```
   cd /home/madvib/dev/ship-worktrees/missing-primitives
   find packages/components/primitives/src/sui-{tag-input,combobox,autocomplete,date-picker,color-picker} -name "*.sui" \
     | while read f; do timeout 5 cargo run --quiet -p sui-lint --example check -- "$f" 2>&1 | grep -E "css-literal|prop-value" ; done
   ```
   Should produce zero violations — both rules are at Error severity.

4. **Primitive registry regenerated.**
   `cargo run -p sui-lint --example regen_primitives -- --check` must
   succeed (no drift between live `.sui` declarations and the on-disk
   `crates/sui/sui-compiler/primitives.json`).

5. **Whole-tree compile passes.**
   `timeout 90 cargo test -p sui-compiler --test shape_conformance --quiet`
   — must pass. This confirms every primitive (the new five plus the
   existing 94) compiles and typechecks against the runtime .d.ts.

6. **Each primitive's vitest spec exists and passes.**
   ```
   cd packages/components/primitives
   timeout 60 npx vitest run \
     test/sui-tag-input.test.ts \
     test/sui-combobox.test.ts \
     test/sui-autocomplete.test.ts \
     test/sui-date-picker.test.ts \
     test/sui-color-picker.test.ts
   ```
   Each spec should at minimum assert registration, shadow root,
   default slot wiring, and reflected-attribute round-trip.

7. **Token discipline.** Spot-check each `<style>` block for raw
   color literals (`#abc`, `rgb(…)`, named colors) and pixel-spacing
   that should be tokenized. The `sui-css-literal` rule catches color
   hits; spacing isn't enforced yet so it's a manual eye-pass.

8. **Canonical enum vocab.** Each enum-typed `<prop values="..." />`
   uses the canonical sets (`size: xs|sm|md|lg|xl` or a sensible
   subset; `tone:` matches the 8-value canonical ladder; `variant:`
   matches `solid|soft|outline|ghost`). Deviation only when justified.

9. **No raw `<script>` / `<style>` outside the primitive's own
   shadow scope.** Per the authoring contract.

## Architectural context

- The branch is forked from `job/sui-framework`. The `sui-prop-value`
  and `sui-css-literal` lints are both at Error severity — any
  violation in new primitives would fail compile, not just warn.
- Layout primitives `<sui-stack>`, `<sui-cluster>`, `<sui-grid>`,
  `<sui-center>` are available; primitive authors should compose them
  instead of writing bespoke flex CSS.
- Active ADRs: none specific to this job.

## Acceptance summary the gate must answer

For each of the 5 primitives, report one of:
- **PASS** — meets all checks
- **PARTIAL** — directory + template exist, but X (specific gap)
- **MISSING** — directory absent
- **BROKEN** — present but fails compile / lint / test

Plus the cross-cutting checks (registry regen, shape_conformance,
overall vitest run).

## web-qa note

UI surface, so the gate is normally web-qa-blocked. The new primitives
are leaf components — they need component-level QA via vitest (which
this spec already requires), not full surface-level playwright QA. If
you need to capture interaction evidence (keyboard nav working, popover
flipping, etc.), drop screenshots into
`.artifacts/web-qa/missing-primitives/`. Otherwise the vitest output
is sufficient evidence for these.

## Report shape

Write the verdict to `.artifacts/gate-missing-primitives.md` in the
gate's own worktree. Single page. Per-primitive table, cross-cutting
checks, top failures (if any), and a one-line overall: ship / ship
with caveats / hold for fixes.

---

> Execute this gate yourself. Read-only inspection of the
> `/home/madvib/dev/ship-worktrees/missing-primitives/` worktree plus
> running the verification commands listed above. Do not modify the
> branch under review.
>
> **When the gate is complete:**
> 1. Write the report to `.artifacts/gate-missing-primitives.md`.
> 2. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 3. Call `mcp__ship__mesh_broadcast` with
>    `{ "topic": "gate.completed", "workspace_id": "<id>",
>      "summary": "<one-paragraph>" }` — skip if shipd is offline.
