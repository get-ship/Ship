# Framework Issues & Data Requests

**Operating principle.** We build the hub on the RPC surface **we have**. We do *not*
change the RPC surface to make ported shell code work — the shell was built on a
now-deprecated surface, and "a prior data call is missing → add it to the backend"
is the instinct that has wrecked frontend work before. When a surface has a
*legitimate* need for an operation or data that isn't available, it is logged here
as a **REQUEST** for the operator to validate — never silently added.

Two kinds of entries:
- **FW-n — framework bugs.** Real defects in deck/sui/shipd behavior. Fixable; not RPC-surface changes.
- **REQ-n — data/op requests.** A surface wants an op/shape not on the current contract. **Blocked pending operator validation. Do not add to the backend to satisfy ported code.**

---

## FW-1 · deck router: static routes mis-route to `:param` siblings
- **Symptom:** `/studio/agents/create` resolves to the `:id` detail view (mounts the wrong controller → was the source of the `agent is not defined` crash).
- **Cause:** `crates/deck/deck-bundler/src/routes.rs:871` sorts patterns lexicographically (`a.pattern.cmp(&b.pattern)`); `:id` (0x3A) sorts before `create` (0x63), and the router (`router/src/index.ts:353`) first-matches without specificity.
- **Blast radius:** any static route that is a sibling of a `:param` route — not just create.
- **Fix:** sort static segments before dynamic, or match by specificity.
- **Location:** **sui repo** (deck-bundler + router). **Status: blocked on sui access (operator ping).**

## FW-2 · component catalog gathers by relative path (dead end)
- Old `workspace.json` / `/_deck/catalog` walked the workspace by relative path; broken cross-repo and in prod. The **registry** is the sanctioned replacement (no relative-path component gathering). Investment when ready; `/studio/components` stays empty until then.

## FW-3 · Does deck belong in the live-data loop at all? (DataPort may be redundant)
- **Today's asymmetry:** unary reads go SUI → `ctx.clients.ops` (Connect client) → shipd **directly, no deck**. Live reads go SUI `live()` / `<data live>` → deck DataPort → `/_deck/mux/subscribe` → shipd resolvers (deck **mediates**) — and *that* mediation is the entire source of REQ-1..6: every live op must have a deck-mux resolver registered.
- **Hypothesis:** SUI can consume live data **directly from the SDK** too — a Connect **server-streaming** RPC fed into `live()` — with no deck mux and no per-op resolver. Then "unary vs live" collapses to "Connect unary vs Connect server-streaming," both straight on the contract, deck out of the data path. deck keeps the value it actually has: SSR of untrusted code, routing, bundling, the component/view system.
- **What to weigh:** the mux's single-connection multiplexing vs Connect-over-HTTP/2's native stream multiplexing. If the transport is HTTP/2, the mux is likely redundant; on HTTP/1/SSE it earns its keep. Transport + framework call is the operator's.
- **Consequence if deck exits the data loop:** REQ-1..6 reframe to "add a Connect streaming method to the contract," with no deck resolver layer to maintain.
- **Decision: operator.** Not to be pre-answered by code.

---

## REQ-1 · agent detail / settings / wire / create-context data  — RESOLVED (unary)
- **Surfaces:** `/studio/agents/[id]`, `[id]/settings`, `[id]/wire`, `create`.
- **Resolution:** the agent-detail data IS on the contract — as **unary Connect** methods on `AgentsService`, not live resolvers. The surfaces read `ctx.clients.ops.agents.{wire,settings,createCtx,get}` wrapped in `live(() => …)` (the proven `ops.collections.get` pattern: a promise drives the `{#async}` loading/error/ok lifecycle). Bindings were re-mapped from the deprecated live shape (snake_case serde JSON) to the proto-es shape (**camelCase**: `mcpServers`, `pathD`, `toolPatterns`, `permissions.toolsAllow/toolsAsk/toolsDeny`). `CreateCtx` returns a `JsonResponse` ({ json }) decoded in the controller; its payload is serde snake_case (`templates`, `permission_presets`, `available_harnesses` — note: **`available_harnesses`**, not `available_providers`).
- **Projection gap (observation, not a blocker):** the canonical `AgentDetail` projection (`ship-agents::view_state::AgentDetail`) carries **no `providers`, `default_provider`, or `hue`**, and the wire builder emits no `prov` nodes. Detail + settings source `providers`/`hue` from the **raw doc** via a second `agents.Get` read (parse `docJson`). A future pass could fold `providers`/`hue` into `AgentDetail` so the providers section + wire-inspector work from a single read (no second round-trip). Not changing the backend to do so — logged for the operator.
- **Note:** `create` cannot be render-verified — `/studio/agents/create` still mis-routes to `[id]` per **FW-1**. The op + decode are correct and compile.

## REQ-2..5 · feature surfaces call live ops not on the contract — NEEDS OPERATOR VALIDATION
Every hub *detail/feature* surface (ported from the shell) reads via a `ship.*` **live**
op that has no registered resolver. The list/overview surfaces (`agents.summaries`,
`collections.list`, `account.state`, `git.overview`) are the whole available live surface.
`ops.*` are unary Connect calls (mutations mostly) and are a separate question.

| Surface | live read it calls | on contract? | unary `ops.*` it also uses |
|---|---|---|---|
| `studio/agents/[id]` (+settings/wire/create) | `ship.agents.wire`, `agents.create_ctx` | ❌ | `ops.agents.patch` |
| `studio/collections/[id]` | `ship.collections.detail` | ✅ **RESOLVED (unary)** | `ops.collections.get` (read) + `ops.collections.{createItem,updateItem}` |
| `studio/pipelines` (+`history`, `run/[slug]`) | `ship.pipelines.{summaries,history,run}` | ✅ **RESOLVED (unary; focus + live-tail deferred)** | `ops.pipelines.{summaries,list,history,run,dispatch,put,cancel}` |
| `activity` | `ship.mesh.snapshot` | ❌ | — |
| `settings` | `ship.settings.update` | ✅ **RESOLVED (unary)** | `ops.settings.{list,watch,set}`, `ship.account.state` ✅ |

**Operator decisions (do NOT pre-answer by editing the backend):** for each — is the surface
in scope on the *current* contract? Should the read be **unary** (`ops.*`, request/response)
rather than live? Or is it a genuine new-op request? The ported code assumes the deprecated live shape.

**Resolution (collections/[id], settings, agents-list run-status — same pattern as REQ-1):**
the data IS on the contract as **unary Connect** methods, read through `ctx.clients.ops`
wrapped in `live(() => …)` so the promise drives the `{#async}` lifecycle.
- `studio/collections/[id]`: read `ops.collections.get({ id })` → `CollectionDetail { id, schemaJson: bytes, itemCount }`. The schema arrives as JSON bytes (decode + project to `{ schema: { id, name, fields, builtin }, total_items }`, same decode the collections LIST controller does). Mutations are the camelCase Connect writes `ops.collections.{createItem,updateItem}({ collection, id?, dataJson })`. **Gap:** `Get` carries schema + `itemCount` only, never the rows (`Query` exposes rows but needs a Kysely AST the frontend doesn't build), so the **Rows tab degrades to its empty state** and a write won't reflect without a reload. A JS-friendly "all items" read (named query, or items on `Get`) would close it. Schema/meta/JSONC tabs + row count are fully populated.
- `settings`: read `ops.settings.list({ scope: RUNTIME })` + `ops.settings.watch` (render trigger); write `ops.settings.set` (the contract verb — the template's `mutation="ship.settings.update"` resolves to a scope closure that calls `set`). `ship.account.state` stays LIVE (on-contract resolver) — verified 200.
- `studio/agents` (list) run-status: the `agentEvents` feed switched from the dead live `ship.events.list({ event_type: 'agent.' })` (no resolver → `mux/subscribe` 404) to unary **`ops.events.poll({ namespace: 'agent', limit })`** → `EventsService.Poll` → `JsonResponse { json }` wrapping `{ events: EventEnvelope[] }` (snake_case fields: `event_type`, `actor`, `created_at`). `namespace` server-side filters to `agent.*`; the JSON is decoded in `agentStatus`. Degrades cleanly: empty feed → empty status map → cards render with no dots.

**Resolution (REQ-3 · studio/pipelines list/history/run — same pattern):** the three core
reads switched from the dead live `ship.pipelines.{summaries,history,run}` (no resolver →
`mux/subscribe` 404) to the **typed unary** `ops.pipelines.{summaries,list,history,run}`
wrapped in `live(() => …)`. The TYPED projection messages are camelCase proto-es:
`PipelineRunRow { pipelineRunId, slug, status, startedAt, endedAt?, outcome? }`,
`PipelineStepRunRow { stepId, status, startedAt?, completedAt?, dependsOn, stepType, attempt }`,
and `PipelineRunHistoryResponse.{ runs, total, offset, limit }` where the count fields are
**bigint** (uint64 — request `offset`/`limit` pass `BigInt(…)`, response reads `Number(…)`).
The contract DELIBERATELY EXCLUDES the GUI view-model the live stream carried (def
step_count/total_runs/success_rate, run agent/branch/provider/model, providers list,
workspace_id, status_text/tone, step observations, event timeline) — the frontend is
adapted: badge tone + `sui-pipeline-*` state derived from `status`; the definitions section
reads `ops.pipelines.list` (`IndexEntry { id, scope }`, name+scope only); durations computed
from startedAt→endedAt. Mutations stay on the unary writes: `dispatch` (run), `put` (create),
and **`cancel`** (slug-keyed stop — the correct verb; the shell routed to `endTurn`, a likely
no-op). **Deferred (not blockers):** (a) `focus/[name]` — the typed contract has no unary
`focus`; rebuilding it needs `ops.pipelines.get` (raw doc) composed with run reads, so a
definition card opens the Dispatch dialog (pre-filled) instead of a focus route; (b)
**live-tail** — `SummariesLive`/`RunLive`/`Watch` server-streams (the concrete FW-3 case), the
unary snapshot is the read for now; (c) run-detail's live log stream (a REST SSE endpoint,
`/api/v1/pipelines/runs/{slug}/log/stream`, not a proto op). Render-verified (daemon 9392):
list paints 6 real definitions (Summaries+List 200), history a clean empty-state (History 200),
run-detail wired (Run called; a bogus slug renders the `not_found` error-state, no crash);
zero `mux/subscribe`; the only 404s are the pre-existing `pty-sessions` chrome noise (REQ-6).

## REQ-6 · command-bar session-state surface (pty-sessions + agent.session/hook live) — NEEDS OPERATOR VALIDATION
- **Surface:** `<sui-command-bar>` (cmdk), mounted hub-wide. The **core** palette (navigate, `ops.search.search`, `ops.agents.list`, `ops.agents.harnesses`) is fully wired and returns **200** — verified. This REQ is only the *session-state* sub-feature (active-agent dot + AI/chat lifecycle).
- **Calls not on the contract (both degrade gracefully — caught → "offline"/empty; neither blocks the palette):**
  - `GET /api/v1/pty-sessions` → **404**. `loadCurrentSession()` (controller) uses this raw REST path to find the active agent session. The endpoint is not served on the current daemon.
  - `ship.onNamespace('agent.session.', …)` + `ship.onNamespace('hook.', …)` → the live mux (`/_deck/mux/subscribe`) **404s these namespaces** (the route exists — GET is 405 — but `agent.session.*`/`hook.*` have no registered live resolver). Same class as REQ-2..5: a live namespace not on the contract. The controller's own header comment (lines ~185-199) already documents that the per-chunk harness stream is not on the Connect surface today.
- **Question for the operator (do NOT pre-answer by editing the backend):** is the active-session/AI-chat surface in scope on the current contract? If yes, what is the right shape on the *available* surface — a unary `ops.*` for the active session, and which (if any) kernel namespaces are subscribable via the mux? Until validated, the bar shows "No session" and the AI/chat lifecycle stays inert; the navigate/search/agents palette is unaffected.

---

### Notes
- icon-404s: a per-page asset 404 (lucide). Cosmetic; lower priority. Likely framework asset-serving, not a data issue.

---

## CULL TRIAGE — `apps/shell` surface-by-surface (shell→hub port)

The **RPC surface is the source of truth** for which UI surfaces exist. The port is a
**cull**, not a preserve-everything. Decision rule per surface:

- **KEEP** — reads only the on-contract **live** resolvers (`agents.summaries`,
  `collections.list`, `account.state`, `git.overview`) **or** a **unary `ops.*`**
  Connect method that exists in `proto/ship`. RPC-backed + current → port.
- **REQ** — wanted surface whose **live** read is *not* one of the 4 registered
  resolvers. Operator decides: switch to a unary `ops.*` (often already exists) /
  a Connect server-stream (FW-3) / register a resolver. **Do NOT add to the backend
  to satisfy ported code.**
- **DIE** — placeholder UI or stale surface → flag for delete, **do not port**.
  (Shell deletion happens at deprecation, operator-driven — flagged here only.)

**Contract facts that drive the column.** The off-contract problem is the **live
(deck-mux) resolver layer** — only 4 are registered. The unary Connect surface is
much wider: `proto/ship/ops/v1` ships unary methods for agents (incl. `Wire`,
`Settings`, `CreateCtx`, `Detail`, `Summaries`), collections (`Get`/`List`/items),
skills (`List`/`File`/`Summaries`), settings (`Get`/`List`/`Set`), ui (`Packs`/`Read`),
search (`Search`), space (`Add`/`List`/`Activate`), artifacts (`List`), and pipelines
(plus **streaming** `SummariesLive`/`RunLive`/`HistoryLive`). There is **no** `git`,
`mesh`, or `canvas` proto service at all — `git.overview` is a live-only resolver
(on contract); `mesh.snapshot` and `canvas.pair` are live-only **and off-contract**
(genuinely new ops, no unary fallback).

### KEEP

| Surface | deciding op | why KEEP |
|---|---|---|
| `routes/home` | `ship.git.overview` (live) | on-contract live resolver |
| `routes/setup` | `ops.space.add` (unary) | `SpaceService.Add` on the Connect surface |
| `routes/component-viewer/[name]` | `ops.ui.packs`, `ops.ui.read` (unary) | `UiService.Packs/Read` on the Connect surface |
| `routes/studio` (landing) | — (chrome only, no data) | static landing; already ported to hub |
| `routes/studio/agents` (list) | `ship.agents.summaries` (live) | on-contract live resolver |
| `routes/studio/collections` (list) | `ship.collections.list` (live) | on-contract live resolver |
| `routes/studio/components` (+`[name]`) | registry catalog + `ops.search.search`/`ops.ui.*` (unary) | no live dep; catalog empty pending the **registry** (FW-2), not an RPC gap |
| `routes/studio/themes` (+`[id]`) | — (client-side token catalog, no op) | no daemon read |

### REQ — wanted, but the live read is off-contract (operator validates; do NOT pre-answer)

| Surface | off-contract live read | unary fallback on the surface? | tracks |
|---|---|---|---|
| `routes/activity` | `ship.mesh.snapshot` | **none** — no `mesh` proto service exists | **REQ-4** (the wanted cross-space system-view; kept) |
| `routes/canvas` | `ship.canvas.pair.*`, `ship.artifacts.list` | `canvas` has no proto; `ArtifactsService.List` exists (unary) | REQ (new) — also reads `ship.agents.summaries` ✓ |
| `routes/settings` | `ship.settings.update` | `SettingsService.{Get,List,Set,Watch}` exist (unary) | **REQ-5**; reads `ship.account.state` ✓ + `ops.settings.*` ✓ |
| `routes/studio/skills` (+`[id]`) | `ship.skills.summaries`, `ship.skills.detail` | `SkillsService.{List,File,Summaries}` exist (unary) | REQ (new) — wanted draft, kept |
| `routes/studio/collections/[id]` (+`partials`) | `ship.collections.detail` | `CollectionsService.Get` exists (unary) | **REQ-2** |
| `routes/studio/pipelines` (+`history`, `run/[slug]`) | `ship.pipelines.{dispatch,history,run,summaries}` | ✅ **RESOLVED (unary)** — `ops.pipelines.{summaries,list,history,run,dispatch,put,cancel}`; `focus/[name]` + live-tail deferred (FW-3) | **REQ-3** |
| `routes/studio/agents/[id]` | `ship.agents.wire` | `AgentsService.Wire` exists (unary) | **REQ-1** |
| `routes/studio/agents/[id]/settings` | `ship.agents.wire`/settings | `AgentsService.Settings` exists (unary) | **REQ-1** |
| `routes/studio/agents/[id]/wire` | `ship.agents.wire` | `AgentsService.Wire` exists (unary) | **REQ-1** |
| `routes/studio/agents/create` | `ship.agents.create_ctx` | `AgentsService.CreateCtx` exists (unary) | **REQ-1** |

### DIE — placeholder/stale → flag for delete, do NOT port (shell: flag only, not deleted here)

| Surface | deciding signal | note |
|---|---|---|
| `routes/account` | pure placeholder UI — disabled "Connect account" button, copy "Not connected yet; sign-in lands once the account service is wired" | `account.state` is on-contract but returns `logged_out` only; no getship.dev account API → no real surface. **Not ported to hub.** |
| `routes/library` | pure placeholder UI — disabled "Browse marketplace" button, copy "Lands once the pack registry is wired" | no op at all; awaits a pack registry. **Not ported to hub.** |

### Chrome / cross-cutting

- `<sui-command-bar>` (hub-wide) — core palette is on-contract & verified 200; only the
  session-state sub-feature is off-contract. Tracked as **REQ-6** above.
- **Stale `'mesh'` glyph entries** (flag, shell-only): `apps/shell/src/components/sui-view-picker/{data,render}.ts`
  and `apps/shell/src/components/sui-launcher/render.ts` carry `'mesh'` in their
  `HIDDEN_VIEWS` / `KNOWN_APP_NAMES` lookup lists, referencing a `mesh` *view* that has
  no route anywhere. Harmless leftover (the view list `fetchViews()` returns empty), but
  stale — drop `'mesh'` (and the other dead names) from those lists at shell cutover.
- `apps/shell/src/chrome/open-space.ts` is live (used by the shell); its hub port
  (`apps/hub/src/chrome/open-space.ts`) is **unreferenced** — see cull notes (flagged, not deleted).
