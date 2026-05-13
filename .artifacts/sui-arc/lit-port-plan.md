# Lit → SUI port plan

Status: draft for approval. Inventory taken on `feat/ship-set` after the
sui-alert-dialog + sui-select ports landed.

## Method

- Source: every file under `packages/ship-set/src/` that calls
  `customElements.define('ship-*')`. Total 103 tags across 92 LitElement
  classes (some files register multiple tags).
- Target: every `.sui` file under `packages/ui-components/src/`. Total 90
  tags.
- Diff is file-level (tag-level), not directory-level — earlier counts
  inflated by directory mismatches.

## Tag-to-file collapsing

Several gap tags share a single Lit source file. Real port effort is
fewer files than tags suggests:

| Gap tags | Source file (Lit) | Ports as |
|---|---|---|
| navigation-menu, navigation-item | ship-navigation-menu.ts (261 LOC) | one sui-navigation-menu/ dir, two tags |
| git-diff, git-diff-file, git-diff-hunk, git-diff-line | ship-git-diff.ts (342 LOC) | one sui-git-diff/ dir, four tags |
| tab-panel | ship-tabs.ts (already partially ported) | add to existing sui-tabs/ |

After collapsing, ~22 distinct files map to the 33 gap tags.

## Apparent gaps that are actually naming mismatches

Worth investigating before porting:

- **ship-tree-node ↔ sui-tree-item.** Same concept. Pick a name and
  consolidate. Recommend `tree-item` (already the SUI name; matches
  `accordion-item`, `breadcrumb-item`, `rail-item` convention).
- **ship-progress-ring vs sui-progress.** sui-progress is linear; ring is
  circular. Real gap, ports as `sui-progress-ring` in the existing
  `sui-progress/` dir (compound co-location, mirroring sui-toast/).
- **ship-step-segments vs sui-stepper.** Need feature diff. If
  step-segments is just a visual variant of stepper, fold in as a
  `variant=` prop on sui-stepper instead of a new component.
- **ship-view-chrome vs the SUI host shell.** May be host-specific
  furniture, not a general primitive. Investigate before porting.

## Grouping by domain

### Group P — Primitives (generic, high reuse)

Highest priority. Drop-in primitives any agent-authored app may want.

| Tag | Lit LOC | Notes |
|---|---|---|
| file-upload | 196 | Drag/drop + progress. Generic. |
| scroll-area | 214 | Custom scrollbars (Radix shape). Generic. |
| navigation-menu (+item) | 261 | Accessible nav, two tags one file. |
| tab-panel | (inside ship-tabs) | Small. Folds into sui-tabs/. |
| step-segments OR fold into sui-stepper | 85 | Decide via feature diff. |
| view-chrome | 197 | Decide if generic or host-specific. |

Total Group P: ~5-6 files. Each is one porter agent.

### Group V — Viz (charts, shared util)

| Tag | Lit LOC |
|---|---|
| area-chart | 114 |
| bar-chart | 108 |
| line-chart | 115 |
| donut | 106 |
| gauge | 117 |
| heatmap | 115 |
| meter | 95 |
| progress-ring | 94 |
| sparkline | 93 |

Plus `viz/util.ts` (shared chart helpers). Total ~1000 LOC across 9 files.

Best ported as a batch — one agent owns the shared util port first, then
parallel agents port individual charts using it. SVG-heavy, suits SUI's
template model well.

### Group D — Domain (app-internal)

Lower priority — app-internal value, not agent-app primitives.

| Tag | Lit LOC | Notes |
|---|---|---|
| git-diff (+file +hunk +line) | 342 | Multi-tag, one file. Compound. |
| branch-row | 127 | Row variant; could be sui-list-tile preset. |
| file-row | 119 | Same as above. |
| history-row | 210 | Row variant. |
| pipeline-dag | 49 | Graph viz. |
| pipeline-definition | 96 | App-specific. |
| pipeline-run | 119 | App-specific. |
| pipeline-step | 137 | App-specific. |
| pipeline-step-def | 175 | Schema helper, may not register a tag. |
| auth-dialog | 324 | Specialization of sui-dialog. |
| provider-blob | 129 | Provider logo. |

Total Group D: ~10-11 files. Recommend porting on-demand when a specific
app needs them, not as a sweep.

### Group E — Editors (island bundles)

| Tag | Lit LOC | Notes |
|---|---|---|
| code-editor | 412 | Monaco wrapper. Heavy. |
| markdown-editor | 302 | Lexical/CodeMirror wrapper. Heavy. |

These ship as island bundles (not direct `.sui` compiles). Port as new
entries under `packages/ui-components/src/islands/`. Last in sequence —
they need their own ESBuild config + island host wiring.

## Recommended sequencing

```
Phase 1 (now)  — form-value directive + sui-tab consolidation (in flight)
Phase 2 (next) — Group P primitives (5-6 ports, parallel agents)
                 Investigate the 4 naming-mismatch items first; consolidate
                 where appropriate before porting.
Phase 3        — Group V viz: shared util.ts port, then 9 charts in parallel
Phase 4        — Group D domain: port on-demand (no sweep)
Phase 5        — Group E editors: island bundles (separate sub-pattern)
```

## Per-port budget

- Group P: 1 agent per component, ~30 min each. ~3 hr total for the 5-6.
- Group V: 1 agent for util + 1 per chart. ~30 min each. ~5 hr for 9 charts.
- Group D: defer; budget per-app.
- Group E: ~4 hr each, sequential (island bundle config is fiddly).

## Open questions for human

1. **tree-node vs tree-item naming.** Reconcile before porting more compound
   parts. SUI convention is `*-item`; recommend ship-tree-node migrates to
   `sui-tree-item` (already exists; check feature parity).
2. **view-chrome scope.** Is this a general primitive (port it) or host
   furniture (skip, treat as legacy Lit)?
3. **step-segments vs stepper.** Fold into existing or new component?
4. **Editors as islands?** Confirm we want them as `.sui` islands rather
   than Lit-stays-Lit.
5. **Domain ports.** Should we port any speculatively, or strict on-demand?

## Cross-cutting work first

The two in-flight tasks unblock Group P:

- **form-value directive** (in flight) — file-upload, scroll-area both
  participate in forms. Easier port once the directive lands.
- **sui-tab consolidation** (in flight) — establishes the file layout
  Group P ports should mirror.

After both land, Group P can start with consistent infrastructure.
