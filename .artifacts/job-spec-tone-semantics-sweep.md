# Job Spec: tone semantics sweep — normalize warning/destructive/info usage across views

**Branch:** `job/tone-semantics-sweep`
**Agent:** ship-designer
**Mode:** autonomous
**Reference:** Track 2 audit finding #2 from this session — *"Tone semantics aren't a system. `tone='warning'` means 'editing state' in pipelines, 'validation issues' elsewhere; `tone='destructive'` for save errors but `tone='warning'` for validation issues."*

## Goal

Establish a canonical tone map and sweep views to use it consistently. Today the same tone is used for divergent meanings across surfaces — this hurts the design system's coherence. A user shouldn't see `tone="warning"` mean "in progress" on one screen and "validation issue" on another.

## Tone map

| Tone | Semantic |
|---|---|
| `default` | Neutral, unscoped (no signal) |
| `primary` | Positive emphasis, current selection, "this is the brand" |
| `info` | Informational notice, in-progress state, "you might want to know" |
| `success` | Completion, "this worked," validation passed |
| `warning` | Validation issue, "this needs your attention," recoverable |
| `destructive` | Error, irreversible action, "this won't work" |
| `accent` | Decorative emphasis, not semantic |
| `muted` | Deemphasized, "this is here but quiet" |

## File scope

```
views/agents/**/*.sui                 # sweep
views/agents/**/*.ts                  # if controllers map state → tone
views/skills/**                       # sweep
views/pipelines/**                    # sweep — heaviest cleanup expected (per audit)
views/sessions/**                     # sweep
views/events/**                       # sweep
views/git/**                          # sweep
views/home/**                         # sweep
views/collections/**                  # sweep
views/canvas/**                       # sweep
views/setup/**                        # sweep
views/specs/**                        # sweep
views/settings/**                     # sweep
views/mesh/**                         # sweep
views/artifacts/**                    # sweep
views/theme-studio/**                 # sweep
views/ship-ui-studio/**               # sweep
apps/shell/**/*.sui                   # sweep
```

DO NOT touch:
- `packages/components/**` — primitive defaults are settled (audit pass already landed). This sweep is about consumers, not primitives.
- Sub-route polish jobs (parallel) — coordinate via commit ordering.

## What to change

For every `tone="X"` attribute or `prop:tone={X}` binding across the view layer:
1. Identify what state the tone is communicating (in-progress vs validation issue vs error vs success vs decorative).
2. Re-map to the canonical tone from the table above.
3. Where a tone is decorative (visual emphasis, no semantic meaning), use `tone="primary"` or `tone="accent"` based on whether it's "brand" or "color flair."
4. Where the same semantic appears with two tones on different surfaces (the audit's main complaint), normalize.

### Specific known offenders (per audit)

- `views/pipelines/focus/[name]/page.sui:130-131` — `tone="warning"` for "editing" state → should be `tone="info"`
- `views/pipelines/focus/[name]/page.sui:154-158` — chip warning + chip destructive in same cluster → reconcile semantically
- `views/skills/page.sui:142-143` — `tone="destructive"` for save errors (OK), `tone="warning"` for validation warnings (OK, but ensure consistency)
- `views/setup/page.sui` — mixed `tone="destructive"` and `tone="warning"` for form errors → use destructive for blocking, warning for advisory

## Architectural context

- Charter rules apply.
- This is a mechanical normalization with judgment calls per site. Where unclear, prefer the canonical mapping above and document the reasoning in the commit.
- Do NOT introduce new tones. The eight-tone palette is closed.

## Acceptance criteria

1. Every `tone="X"` on a primitive in the view layer maps to a canonical meaning.
2. No surface uses contradictory tones for the same semantic (cross-view consistency).
3. The canonical tone map (table above) is documented in a comment at the top of `views/README.md` (create if absent) OR in a new `.artifacts/tone-semantics.md` reference doc.
4. `bun run build` passes.
5. `bun packages/deck/bundler/index.ts --all` succeeds.
6. View test suites pass (any test asserting tone behaviors stays green).
7. Visual QA artifact at `.artifacts/web-qa/tone-semantics-sweep/<timestamp>/`. Cover at least 3 surfaces where tones changed (pipelines focus, skills, setup). Compare before/after.

## Out of scope

- Primitive default tones (already settled by audit pass).
- New tones / tone catalog changes.
- Per-project theme tones (orthogonal — themes can shift hue palette, not semantic meaning).

---

> Execute. Mechanical sweep. One large commit OR per-view commits both fine.
>
> When done: complete_workspace + mesh_broadcast.
