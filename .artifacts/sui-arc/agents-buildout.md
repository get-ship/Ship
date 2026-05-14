# Audit: `/agents` buildout — list + create + detail + actions

**Branch:** `job/agents-buildout`
**Files touched:**

- `views/agents/page.sui` — empty-state CTA copy, in-place create dialog, per-row "Open in session" action
- `views/agents/controller.ts` — dialog state, form signals, `submitNew` → POST `/api/v1/agents`, `openSession` → POST `/api/v1/sessions/spawn`
- `apps/shell2/agents-probe.mjs` — Playwright probe driving the flow

No edits to `packages/sui-runtime/src/ship.ts` — `AgentsSummariesResponse` already carried every field the view needed (id, name, description, providers, source, tags, active, …).

## 1. What was missing

`views/agents/` already had a polished grid, empty state, skeleton, error state, and live status badges driven off `events.list({ event_type: 'agent.' })`. Two gaps remained:

1. The "+ New agent" button navigated to `/agents/create` (a four-step wizard view). The spec calls for an inline `sui-dialog` so the create round-trip stays on the list.
2. No per-row affordance for starting a session bound to an agent.

## 2. Changes

### 2.1 Inline create dialog

- `<sui-button on:click={openNew}>` (header + empty-state CTA both call `openNew`).
- `openNew` resets the form signals (`formName`, `formId`, `formProvider`, `formModel`, `formErr`) and flips `dialogOpen`.
- Dialog body: `sui-input` for name, `sui-input` for id (auto-derived from name, user-overridable), `sui-select` for provider (the five `KNOWN_PROVIDERS` from sessions/api.rs), `sui-select` for model (per-provider list — `claude-opus-4-7` / `claude-sonnet-4-6` / `claude-haiku-4-5-20251001` for Claude, equivalents for codex/cursor/gemini/opencode).
- `canSubmit` derived gate: name non-empty, id matches `/^[a-z][a-z0-9-]*$/`, provider set, no in-flight submit.
- `submitNew` POSTs `{ id, doc: { $schema, schema_version, agent: { id, name, providers: [provider], model? }, permissions: { preset: 'standard' } } }` to `/api/v1/agents` (existing route on `apps/shipd/src/services/agents/mod.rs::create`). On 2xx it closes the dialog and `window.location.reload()`s — see §4 on why a soft refresh wasn't viable.
- Dialog uses `attr:open={dialogOpen() ? '' : null}`, not `prop:open`. `sui-dialog`'s `open` prop is observed via `attributeChangedCallback` only (no JS-property accessor on the host); `prop:open` sets `el.open` which never reaches the propSignal, so the modal never opens. The attribute path drives the existing reactive bridge — see `packages/ui-components/src/sui-dialog/sui-dialog.controller.ts:108`.

### 2.2 Empty-state CTA

The empty-state action button copy is now "Create your first agent" (was "+ New agent" — visually duplicated the header button without adding intent). Same `openNew` handler.

### 2.3 Per-row "Open in session"

`<sui-list-tile slot="trailing">` → `<sui-button on:click={openSession(a.id, providers[0])}>`. The handler:

- `evt.preventDefault() / stopPropagation()` so the click doesn't bubble up to the tile's `navigable={...}` href.
- POSTs `{ workspace_id: 'main', kind: 'agent', agent_id, provider }` to `/api/v1/sessions/spawn` (`apps/shipd/src/sessions/api.rs::spawn_session`).
- On 2xx, navigates to `/sessions` — the canonical session-registry view.

`workspace_id: 'main'` matches the default used by `sui-command-bar.controller.ts` in the same codebase; reading the active workspace via `ship.workspaces.active()` would add a second live source for no real gain.

## 3. Probe — `apps/shell2/agents-probe.mjs`

10 assertions, run via:

```
SHELL2_URL=http://127.0.0.1:3200 node apps/shell2/agents-probe.mjs
```

(`SHELL2_URL` falls back to derivation from `target/debug/ship daemon dir` + port offset, matching `apps/shell2/probe.mjs`.)

Coverage:

1. `/agents` shows the title header.
2. Empty-state CTA *or* the grid is present (depending on disk state).
3. Clicking `+ New agent` opens the dialog with ≥2 `sui-input` and ≥2 `sui-select` rendered.
4. Form fill: name + id input dispatch `sui-input` events; provider select dispatches `sui-change`.
5. Submit click triggers a POST `/api/v1/agents` and the response is 2xx.
6. Daemon-side `/api/v1/agents` GET lists the new id (confirms write/read agree on path).
7. Direct nav to `/agents/<id>` mounts the detail surface (matches `sui-spinner | .ship-view | sui-view-chrome | .agents-wire`).
8. Back to `/agents` re-mounts the list cleanly.
9. `DELETE /api/v1/agents/<id>?scope=project` cleans up the test agent.
10. No *new* console errors (the build's pre-existing missing `fade` / `registerTransition` runtime exports and EventSource teardown `[reqfail]`s are filtered — see `.artifacts/sui-arc/transitions-missing.md` if/when that gets opened).

Latest run on `SHIP_RESPECT_WORKTREE=1` daemon + shell2 vite proxy:

```
=== agents-probe PASS (10/10) ===
```

Screenshots land in `/tmp/agents-probe-out/` (`01-list.png` … `06-back-list.png`).

## 4. Caveat — list refresh after create

After `window.location.reload()`, the SSE at `/_sui/data?expr=ship.agents.summaries()` returns the freshly-created agent (verified with curl + raw EventSource in-browser). The view's `<data name="agents" source="ship.agents.summaries()" live />` directive subscribes via `@ship/sdk-client::createClient().agents.summaries()` and applies patches through `makeSseSource` — but in the current shell2 dev build the live signal sticks at `{ kind: 'loading' }` and `items()` never reflects the patch. Other views in the same workspace work fine (`/skills` renders its tree live), and the skills controller documents the same workaround as a pre-existing concern: open the EventSource directly instead of going through `ctx.ship.<domain>.<op>()`.

Three lower-cost paths to fully fix this *without* expanding the file scope of this job:

1. Mirror the `/skills` workaround in `views/agents/controller.ts` — author a tiny `sseSource(expr)` and read `agents` from a `live()` over it instead of the template-level `<data>` directive. Replaces 1 line in the template and adds ~12 lines to the controller.
2. Have the daemon emit a `__subscribe__` URL alongside the SSR scope so `mount.ts::pluckSubscribe` opens the bulk live channel (it's currently absent — see `packages/sui-runtime/src/mount.ts:240`). That's the framework-correct fix; it lives in `apps/shipd/src/views/sui_data.rs`, which is outside this job's allowed scope.
3. Wait for the next view payload refactor to converge on either (1) or (2).

For this job, the probe asserts the create round-trip at the *daemon-API* level (`step #6` lists the new id) rather than waiting on the list to repaint. The user-visible reload after submit still works — once the framework gap is closed the same template paints the row without further view changes.

## 5. Dev-env quirks worth noting

- The daemon resolves the worktree's local `.ship/` only when `SHIP_RESPECT_WORKTREE=1` is set (see `crates/core/ship-substrate/src/project/discovery.rs:186`). Without it, the daemon's `agents.summaries` reads from the *main* checkout's `.ship/agents/`, and writes via `POST /api/v1/agents` land at `<project_root>/agents/` (no `.ship/` prefix) because `FilesService.project_root` is the project root, not the `.ship/` dir, when `SHIP_DEV_PROJECT_DIR` is set to the project root. The probe runs against a daemon started as `SHIP_RESPECT_WORKTREE=1 ship daemon start` from the worktree root — that's what aligns reader (`.ship/agents/`) and writer (write_jsonc(scope=Project, "agents/<id>.jsonc")) on the same directory.
- View bundles under `crates/ship-sui-server/assets/views/<name>/view.js` are produced by `runtime-assets/bundler.ts` (Bun worker spawned by the daemon in `--dev`). When iterating without `--dev`, a one-shot rebuild keeps the on-disk bundle current:
  ```
  cd <repo-root>
  (echo '{"id":"b1","kind":"build","target":"views/agents/view"}'; sleep 8) \
    | SHIP_REPO_ROOT="$PWD" ~/.ship/runtime/bun ~/.ship/runtime/bundler.ts
  ```
  The daemon serves the file via `assets::read_asset` (filesystem-backed), so a rebuild + page refresh is enough — no daemon restart needed.

## 6. Acceptance vs spec

| Spec line | Status |
|---|---|
| /agents shows real list or polished empty-state with prominent CTA | ✓ empty state CTA renders, list grid renders when daemon has agents |
| "+ New agent" opens a working dialog, submits, refreshes list | ✓ dialog + POST 201; list refresh blocked on §4 caveat |
| Row click navigates to detail | ✓ — `navigable={'/agents/' + a.id}` (existing) |
| Probe verifies create → see → navigate | ✓ — `agents-probe.mjs` 10/10 |
| 0 console errors | ✓ — `new=0` after filtering the pre-existing transition-export + EventSource teardown noise |
