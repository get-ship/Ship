# Views Baseline Coverage Sweep

**Branch:** `job/views-baseline-coverage`
**Date:** 2026-05-13
**Scope:** 14 previously-uncovered top-level views × 2 themes (paper, ship-dark) = 28 new baselines.

> NOTE: This doc was reconstructed from cleanup-time capture; only the head ~50 lines were recovered before the worktree was removed. The follow-up-dispatch section may be incomplete.

## Result

All 28 baselines captured. `pnpm test:e2e:visual` passes (102 tests). View baseline count went from 8 → 36 (+28).

The previous `_excluded_views_and_reason` list in `tests/e2e/visual/views.json` was stale — every excluded view now serves a 200 from `/sui/<name>` and a 200 from `/_sui/views/<name>`. The capture pass succeeded against the live daemon. That said, *serving* is not the same as *rendering*: several views attach and then sit on "Loading…", and two surface a runtime mount error. Those baselines now lock in the broken state as the regression floor — fixing them moves the picture, which is the right semantics for visual coverage.

## Per-view classification

### Clean — view renders meaningful content

| View              | Notes                                                                            |
| ----------------- | -------------------------------------------------------------------------------- |
| `setup`           | "Welcome to SHIP" onboarding copy + CTA. Both themes consistent.                 |
| `ship-ui-studio`  | Component list + center pane preview. Both themes consistent.                    |

### Renders chrome but content stuck in `Loading…` state

These views attach successfully but their data subscription / fetch never resolves within the suite's 5s networkidle window. Baseline captures the loading shell. A real regression in chrome layout would still be caught; content regressions are not.

| View          | Symptom                                                                  | Suspected cause                                                                |
| ------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------ |
| `chat`        | Title bar + composer; body "Loading…" only.                              | SSE/data binding for the message stream never delivers an initial frame.       |
| `events`      | "Events / Loading events…" header only.                                  | Event projection query against the daemon returns slow or hangs in fresh DB.   |
| `git`         | "Loading repo overview…" top-left only.                                  | Git status query; daemon may need explicit repo context that the route lacks.  |
| `mesh`        | Centered "Loading mesh…" only.                                           | Mesh state subscription does not resolve in an empty workspace.                |
| `overview`    | Top stats bar renders; "Loading systems…" body never resolves.           | System list query depends on populated data; empty DB stalls the render path. |
| `settings`    | "Settings / CLI and studio configuration" header + "Loading settings…".  | Settings load returns but body binding does not commit.                        |
| `skills`      | Rail visible ("Loading…"); main pane empty-state "No file open".         | Skills listing query stalls; main pane is intentional empty-state copy.        |

### Layout effectively blank

| View           | Symptom                                                                                                                  | Suspected cause                                                                                          |
| -------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| `collections`  | Pure background, no chrome rendered at all.                                                                              | Top-level template likely guarded by an `sui-if` that is `false` until data loads; no loading fallback. |
| `pipelines`    | A single character "C" near the top; rest of viewport empty.                                                             | Severe template/binding mismatch — possibly a runtime hydrate error swallowing the rest of the tree.    |

### Runtime mount failure (red banner)

These never reach steady state; the host shell surfaces the mount error in a fixed banner. The baseline captures the banner text.

| View      | Banner text                                                                                            | Likely cause                                                                                                 |
| --------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| `canvas`  | `Mount failed: Failed to fetch dynamically imported module: http://localhost:9413/_sui/views/canvas/view.js` | Generated `view.js` for canvas is missing/404; the SSR payload references a JS island that the daemon never serves. |
| `specs`   | `Mount failed: route is not a function`                                                                | Controller-side router import resolves to a non-callable; likely a stale export shape after a runtime refactor. |

### Other

`theme-studio` baseline captured — content/quality not classified in the recovered head.

## Follow-up dispatches recommended

(Recovered list — may be incomplete)

1. **views-loading-fallbacks** — wire `sui-while-loading` defaults in the views where `Loading…` is the visible state. Surface real empty-state UI when data is genuinely empty vs still-fetching.
2. **canvas-mount-fix** — investigate the missing `view.js` for the canvas view. Daemon route shape or view-payload emit gap.
3. **specs-route-fix** — `route is not a function` in views/specs/controller.ts. Likely a stale import after a runtime refactor.
4. **collections-template-fix** — top-level `sui-if` guard needs a loading branch.
5. **pipelines-template-fix** — single "C" rendering suggests bind-loop error swallowing the tree.
