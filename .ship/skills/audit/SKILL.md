---
name: audit
audience: internal
stable-id: audit
description: Per-module architecture audit — citation-backed, layer-appropriate, observation-accreting.
tags: [audit, methodology, architecture]
authors: [ship]
---

# Audit Skill

## When to use this skill

Invoke this skill when you need to audit a single module's public API, events,
wire integration, and layer-direction discipline. The unit of work is exactly
one module path — one crate, one directory, one logical boundary. Whole-codebase
sweeps do not fit in working memory and do not produce citable output. After N
module passes the running observation log (`_observations.md`) surfaces
cross-cutting refactor candidates without any up-front sweep.

## Inputs you need

1. **Module path** — the Rust crate root or directory, e.g.
   `crates/core/ship-platform/src/agents/` or `crates/core/ship-kernel/`.
2. **Layer indicator** — one of `L0`, `L1`, `shared`, `apps`. Determines which
   section shape and which checks apply.
3. **Citation indexes** — the authoritative emit-site and substrate-fn indexes.
   Read these before writing a single row:
   - `.artifacts/specs/op-surface-audit/_indexes/emit-sites.md`
   - `.artifacts/specs/op-surface-audit/_indexes/substrate-fns.md`
4. **Prior audit docs** (optional) — if the module has been audited before, read
   the prior doc to understand what changed.

## The citation contract

> **Every row in audit output cites `file:line` from the codebase. No citation,
> the row drops on read.**

This rule is non-negotiable. It prevents the cascade: audit-pass speculates →
plan-pass inherits → classification trusts → days of cleanup. A reviewer can
drop any uncited row without discussion. If you cannot cite a line, omit the row
or mark it `[needs-verification: no citation]`.

## Dual-side liveness

An event is **live** only when it has both a producer AND a consumer. Checking
only the emit-site misses the class of bugs where an event type is produced but
never consumed (emitter without listener) or defined but never emitted
(consumer-ready / producer-missing). For every event type in scope:

- **Producer count** — how many `emit(...)` call sites actually name this type?
  Walk helper fns if the emit is indirect.
- **Consumer count** — how many `match` arms, `subscribe_to`, or pattern checks
  name this type?

Both must be ≥ 1 for "live." If one side is 0, that is the finding — not an
absence of data.

## No-hallucination guardrail

Specific failure modes observed in prior passes. Check every section against
these before declaring the audit done:

1. **Speculation-mixed-with-extraction** — do not fill "should emit" rows in a
   "currently emits" table. If it isn't in the codebase, it isn't in the row.
2. **Top-down recommendations without bottom-up enumeration** — cite at least
   one caller for every "external caller" claim. Bare-fn grep without qualified
   path over-counts due to same-named fns in other crates.
3. **Force-fit to avoid a verdict** — if a function is ambiguously `dead` vs
   `internal-leaked`, record the ambiguity. Do not clear it by picking one
   arbitrarily.
4. **Reason-template fabrication** — generic phrases (`"trivial accessor"`,
   `"internal helper"`) must be verified against the actual function, not
   applied by pattern-match to function names.
5. **Single-side liveness** — emit-sites only is not a liveness check.
6. **`rg`-misses on indirection** — helper fns like `emit_artifact_event` wrap
   the real emit call. Always walk the helper's call sites to recover concrete
   event types.
7. **Schema/value drift between registries** — `event_types::ALL` may not
   include event constants defined in per-family modules. Check both.

## Separation rule

Extraction and judgment are different jobs. Mix them and extraction is poisoned
by speculation.

- **Extraction** (mechanical, verifiable, no opinions): walk a directory, emit
  rows of `(symbol, file:line, doc-comment-first-line)`. Output is a fact-set.
- **Judgment** (interpretive, bounded sample, anchored in the fact-set): given a
  citation-backed input, classify / verdict / recommend.

Two passes, never one.

## Qualified-path rule for caller counts

Grepping for callers by bare fn name collides with same-named fns in other
crates and over-counts external callers. Always grep with the qualified module
path (`compiler::foo`) AND the bare name, then verify each bare-name hit is not
a different crate's fn. Example traps: `get_harness`, `list_harnesses`,
`strip_jsonc_comments`, `write_atomic` exist in multiple crates with different
definitions.

## Observation tagging conventions

Use the tag vocabulary established across the audit corpus. The authoritative
schema for collection rows is `.ship/collections/architecture-observations.jsonc`.
The JSONL rows represent a partial seeding; many tags appear extensively in the
audit docs but have not yet had all rows written. Do not invent new tags —
use from the established list below.

Collection field constraints:
- **severity**: `info` / `low` / `medium` / `high` / `critical`
- **tag_status**: `candidate` (first sighting) / `confirmed` (2nd sighting) /
  `promoted` (3rd+ sighting, cross-module pattern)
- **status**: `pending` / `under-review` / `signed-off` / `superseded` / `closed`

Established tag strings from the corpus (use verbatim — do not paraphrase):

```
[over-applied-pub]
[dead-emit: <event-name>]
[dead-pub-fn: <fn-name>]
[duplicate-utility: <fn-name>]
[duplicate-internal: <fn-name>]
[duplicate-type: <TypeName>]
[duplicate-internal: parse_scope]
[parallel-entry-point]
[mr-02-wrapper]
[dual-wire-surface]
[wire-coverage: maximal]
[wire-shape-downgrade: <description>]
[trait-bypass-read: registry-macro-direct]
[mr-02 silent-write]
[fixtures-pattern]
[no-patch-rpc]
[mcp-vestige]
[mcp-injection-vestige]
[cohesion-note: <description>]
[layer-direction: shared→L1]
[layer-purity-enforcement]
[silent-mailbox-drop]
[parallel-systems-consolidation]
[architectural-exception: non-event-sourced storage]
[event-type-drift]
[symmetric-DO-pair]
[minimal-auth-surface]
[security: <short-description>]
[file-cap-violation]
[dead-crate]
[layer-misplacement]
[candidate: <description>]
```

Tag new patterns as `[candidate: <description>]` with `tag_status: candidate`.
Do not promote to a confirmed tag without a second module sighting.
When writing a new row, check the JSONL for the highest existing `obs-NNN` id
and increment.

## Per-layer shape pointer

Layer shape determines which sections appear and which checks dominate.

- **L0** — invariant-centric, soundness-heavy. See `layer-l0.md`.
- **L1** — events-and-wire-centric, trait surface vs bypass. See `layer-l1.md`.
- **Shared** — caller graph, library discipline, layer-direction (shared→L1 = violation). See `layer-shared.md`.
- **Apps** — SDK-boundary purity, transport-vs-logic, auth surface, privileged-ops inventory. See `layer-apps.md`.

## Write-back instructions

Every audit pass produces three artifacts.

### 1. Per-module audit doc

Path: `.artifacts/specs/module-audits/<layer>/<module>.md`

Use the canonical 10-section shape for the appropriate layer (defined in the
per-layer sub-docs). Do not customize per module — the uniform shape is what
makes the collection machine-tractable. Carry the header block:

```markdown
> Pass date: <YYYY-MM-DD> · pass scope: <layer> · citation source: <index path>
```

### 2. Observations log entry

Append a `### <module> — <YYYY-MM-DD>` entry to
`.artifacts/specs/module-audits/_observations.md`. Each cross-cutting pattern
gets one bullet per module sighting. The log accumulates across passes; after
3 sightings a candidate tag becomes confirmed and a promoted refactor candidate.

### 3. Collection rows

Add one row per pattern occurrence to
`.ship/collections/architecture-observations.rows.jsonl`.

Row schema (all fields from the collection JSONC — include all required fields):

```jsonc
{
  "id": "obs-NNN",                   // sequential, check existing rows for next N
  "module": "<module-path>",
  "layer": "L0|L1|L2|shared|apps",
  "tag": "[tag-string-verbatim]",
  "tag_status": "candidate|confirmed|promoted",
  "severity": "info|low|medium|high|critical",
  "citation_file": "<path/to/file.rs>",
  "citation_line": "<line-or-range>",
  "description": "<one-sentence citation-backed description>",
  "status": "pending",
  "audit_doc": ".artifacts/specs/module-audits/<layer>/<module>.md",
  "task_id": null,
  "pass_date": "<YYYY-MM-DD>",
  "notes": null
}
```

### 4. README coverage table

Until `xtask gen-audit-readme` (Spec 5) exists, update the coverage table in
`.artifacts/specs/module-audits/README.md` manually: add a row for the module
with pass date, LOC, status.

## Failure mode checklist

Before declaring the audit done, verify each item:

- [ ] Every public API row has a `file:line` citation?
- [ ] Every event-type row has both producer count AND consumer count?
- [ ] Caller counts used qualified-path grep, not bare-name grep?
- [ ] Helper-fn emit sites were walked (not marked "dynamic" and dropped)?
- [ ] Layer-shape matches the layer indicator (L0/L1/shared/apps)?
- [ ] Observation rows added to `.ship/collections/architecture-observations.rows.jsonl`?
- [ ] `_observations.md` appended with this module's entry?
- [ ] No "should emit" rows in the "events emitted" table?
- [ ] No uncited caller counts?
- [ ] File cap respected? (300 soft / 400 hard — split if needed)

## Typical session

```
1. audit crates/core/ship-kernel/
   → produces .artifacts/specs/module-audits/L0-substrate/ship-kernel.md
   → appends to _observations.md
   → adds rows to architecture-observations.rows.jsonl

2. audit crates/core/ship-platform/src/agents/
   → produces .../L1-platform/agents.md
   → observation log now shows whether kernel + agents share patterns

3. After N modules, read _observations.md for refactor surface
```
