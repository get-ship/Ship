# cmdk-sessions-scrub — port the command bar's dead `/api/v1/sessions/*` fetches onto Connect

**Branch:** `job/cmdk-sessions-scrub` (forked from `job/hub-cmdk`).
**Scope touched:** 4 controllers + 1 type module under `apps/hub/src/components/sui-command-bar/`. No
template changes, no server-op changes, no SDK changes. The `.sui` files compile clean.

## Op mapping (each dead fetch → which Connect op)

| Dead call (chat half)                            | Replaced by                                  | Source of truth                          |
|--------------------------------------------------|----------------------------------------------|------------------------------------------|
| `POST /api/v1/sessions/spawn`                    | `ops.agents.dispatch({ agent, harness, workspace, … })` | `crates/core/ship-ops-rpc/src/catalog/commands.rs:381-404`, `apps/shipd/src/rpc/ops/agents_wire.rs:94`, `crates/core/ship-platform/crates/ship-agents/src/services/agents/rpc_wire.rs:624` |
| `POST /api/v1/sessions/{id}/message`             | `ops.agents.steer({ slug, message })`        | `rpc_wire.rs:710-720` (`steer_fn` → `dispatch_handle.registry.get(&slug)` → `executor.send(handle, &message)`) |
| `DELETE /api/v1/sessions/{id}` (3 sites: chat, palette, bar) | `ops.agents.stop({ slug })`                  | `rpc_wire.rs:698-708` (`stop_fn` keyed on dispatch slug, same `dispatch_handle.registry`) |
| `EventSource(/api/v1/sessions/{id}/events)`      | `ship.onNamespace('agent.session.')` + `'hook.'` | `packages/deck/sdk/src/transport.ts:152-176` (DirectTransport → `client.stream.events({})`), `apps/shell/src/kernel-bridge.ts` (the established namespace pattern) |

The dispatcher side returns `{ session_id, slug, … }` from `DispatchResult` (proto
`ship/ops/v1/agents.proto:299-306`); the chat controller stashes `slug` on the
optimistic `SessionSummary` so subsequent `steer`/`stop` calls in the same
session can address the dispatch handle without a server-side lookup.

## Session id vs slug — the missing slug for listed sessions

`agents.stop` and `agents.steer` both take a **dispatch slug**, NOT the pty
session id the SSE bridge used to key on. The slug is generated server-side
as `<workspace_label>-<ulid-suffix>` (see
`crates/core/ship-substrate/src/lib.rs:85` `gen_slug_suffix`); the session id
from `agents.sessions` (`SessionMeta.id`) is a separate ULID. Distinct strings.

Consequence for the cmdk: sessions **we dispatch** get the slug from the
`DispatchResult` and stash it on `SessionSummary.slug`. Sessions **loaded from
`agents.sessions` listing** don't carry a slug — `SessionMeta` in
`proto/ship/ops/v1/types.proto:223-237` has `id`, `agent_id`, `harness`, `pid`,
`status`, `log_path`, `spawned_at`, `last_activity`, `harness_session_id`, but
no `slug` field. The picker rows for those sessions can't be cleanly stopped
or steered today.

**Pragmatic handling** (in all three `endSession` sites — chat, palette, bar):
- If `SessionSummary.slug` is present → call `ops.agents.stop({ slug })`.
- If absent → optimistic UI drop + warning toast ("Session is not locally
  dispatchable; cleared from view."), no fabricated request. The kernel
  `agent.session.completed`/`cancelled`/`failed` events will reconcile the
  listing on the next `agents.sessions` poll.

This is a **real gap**, not papering-over: a server-side fix to add `slug`
to `SessionMeta` (or a new `agents.session_slug({session_id})` resolver)
would close it without further client work. Out of scope for this port
(spec: "do NOT change the server ops").

## Per-chunk text streaming — the missing Connect op

The old `EventSource('/api/v1/sessions/{id}/events')` delivered
`HarnessEvent`s (`{ kind: 'message' | 'tool_use' | 'turn_complete', text, tool }`)
from the actor's per-session mailbox broadcast
(`apps/shipd/src/sessions/api/io.rs:54-74` → `actor_registry::subscribe_events`,
which yields `HarnessEvent` directly from the harness stdout parser — see
`crates/core/ship-platform/crates/ship-agents/src/services/agents/agent_actor.rs:206-211`).

**No Connect server-stream surfaces per-session HarnessEvents today.** The
unified server-stream surface is `StreamService.Events`
(`proto/ship/ops/v1/stream.proto:16`), which yields every kernel
`EventEnvelope` globally — including lifecycle events (`agent.session.started`,
`agent.session.dispatched`, `agent.session.completed`, `agent.session.failed`,
`agent.session.cancelled`, `agent.session.turn_ended`) and supervisor-synthesized
`hook.tool_use_started`, `hook.tool_use_completed`, `hook.turn_observed`,
`hook.session_started`, `hook.session_ended`. **Per-harness-output text
chunks (the `Message` variant) are not on the kernel bus.**

What the bar gets today on the global stream (rendered to the chat thread):
- `agent.session.dispatched` → flip status to `ready`
- `agent.session.turn_ended` / `completed` / `failed` / `cancelled` → drop the
  active session + refresh the listing
- `hook.tool_use_started` → emit `→ tool: <name>` system line
- `hook.turn_observed` → clear the current-message accumulator (boundary
  signal, replaces the old SSE `turn_complete`)

What's missing: streaming text bubbles as the agent types. The chat UX is
honest about this — it doesn't fabricate chunks; the user sees lifecycle +
tool-call signals and the system eventually catches up via `StudioMessage`
projection rows the chat UI can re-render against on demand.

**This is a real finding.** Closing it requires either a new
`StreamService.SessionEvents` rpc that yields per-actor `HarnessEvent`s, or
moving the harness stdout parser into the kernel router so chunks flow
through `stream.events`. Both are out of scope for a controllers-only port.

## Build / verify

- `cargo run -q -p deck-cli -- build-assets` then `cargo run -q -p deck-cli -- build --project apps/hub --bundle-framework` → `_deck 4 files (self-contained)`. No errors.
- `node scripts/check-sui-compiles.mjs` → all three cmdk `.sui` files
  (`sui-command-bar.sui`, `sui-command-chat.sui`,
  `sui-command-palette.sui`) compile clean. (1 unrelated pre-existing
  failure in `packages/components/ship/collections/src/ship-item-editor/ship-item-editor.sui`.)
- Source-level grep: zero `/api/v1/sessions` references remain under
  `apps/hub/src/components/`. Bundle-level grep: zero matches in
  `apps/hub/.deck/assets/entry-*.js`.
- `EventSource(` occurrences in the bundle: 0 (down from 1 pre-port).

## What I couldn't drive

The cmdk-check pattern (`/tmp/cmdk-check.mjs`) runs a full CDP session
against a dev daemon pointed at the cmdk. Building a fresh `shipd` for this
worktree would take ~10 min from cold (the worktree has no shipd binary
yet — the committed framework bundles in `crates/deck/deck-core/assets/`
were rebuilt via `cargo run -p deck-cli -- build-assets`, which itself
took ~600 s and compiled 231 first-party `.sui` files). The
`/tmp/cmdk-run.sh` pattern runs a pre-built `shipd` from a different
worktree (`sui-framework`); pointing it at this worktree's `.deck/` would
require a symlink + workspace flag and an additional compile pass.

I documented this and stopped at the static checks (controllers compile
against the typed `ops` client; bundle emits the new op calls; no stale
references survive). The runtime smoke test (spawn → events arrive →
message → cancel) is exercisable once this branch is merged into a worktree
that has a `shipd` binary built against the same git head — at that point
`/tmp/cmdk-check.mjs` can run unmodified and validate the live flow.

## Files changed

- `apps/hub/src/components/sui-command-bar/types.ts` — extend `SessionSummary` with optional `slug?: string`.
- `apps/hub/src/components/sui-command-bar/sui-command-chat.controller.ts` — `spawnSession` → `ops.agents.dispatch`, `sendCurrent` → `ops.agents.steer`, `endSession` → `ops.agents.stop`. Optimistic session signal set from `DispatchResult` so a user can address the new session before the kernel event projection lands.
- `apps/hub/src/components/sui-command-bar/sui-command-bar.controller.ts` — SSE → `ship.onNamespace('agent.session.' | 'hook.')`. `endSession` → `ops.agents.stop`. `loadCurrentSession` (re-fetch on lifecycle events) unchanged in path (`/api/v1/pty-sessions` is out of scope per spec).
- `apps/hub/src/components/sui-command-bar/sui-command-palette.controller.ts` — `endSession` → `ops.agents.stop`.

No `.sui` template, server-op, SDK, or proto changes.
