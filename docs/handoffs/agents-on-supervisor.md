# Handoff — `feat/agents-on-supervisor`

Last commit on push: `70976c93`. All work pushed to `origin/feat/agents-on-supervisor`. **Not** merged to main since the components need browser verification.

## What landed this session

Commits since `687831ce` (the last thing on main when we started this segment), in order:

1. `8158d5a0` — provider parity: `persistent_session` + `push_mechanism`. All 5 providers now run persistent sessions; mechanism is data not behavior.
2. `205eb12a` — mesh substrate: `/proc` reader, `/api/v1/mesh/snapshot`, `kill-tree` primitive.
3. `b2e95e4b` — mesh SSE diff stream `/api/v1/mesh/events` (1.5s tick).
4. `4e7d6dc7` — per-actor `msg_rate` counter + `/api/v1/*` prefix sweep.
5. `a06f5118` — per-turn `studio.reply` parity for non-Claude providers.
6. `b4fa4af3` — long-lived `studio.reply` parity for non-Claude providers.
7. `55884a7c` — mesh system bar — total msg_rate, total rss, platform.db size.
8. `28d32e1e` — Justfile freshness pass (delete dead apps/web recipes; add `dev-daemon`, `daemon-logs`, `repair-migrations`, `repair-migrations-mark`).
9. (post-pull) — merged `1898f097` from `main` (SFC framework + ship-components skill + `.ship` LSP recognition).
10. `7db21b74` — phase-1 chat-modal SFCs (modal + thread + composer) + Vite proxy for `/_ship` + dev-daemon.sh target-dir resolution fix.
11. `c30f87fd` — phase-1.5 PTY interactive attach (`<app-pty-attach>` + `ChildHandle::resize` + `POST /api/v1/sessions/{id}/resize`).
12. `7db21b74` — phase-3 annotation backend (`annotations[]` body extension, XML `<context>` render-into-prompt, `studio.chat_turn` payload extension with `text` / `raw_text` / `annotations`).
13. `70976c93` — phase-2 picker + phase-5 pipeline board (`<app-agent-picker>`, `<app-pipeline-step>`, `<app-pipeline-board>`, `.ship/views/pipelines-board/`).

5/5 loops green throughout.

## SFC components shipped (in `.ship/components/`)

| File | Purpose | Browser-verified |
|---|---|---|
| `app-agent-modal.ship` | Top-level chat modal; ⌘J toggles; listens for `ship:chat:*` window events | ❌ |
| `app-agent-picker.ship` | Agent + provider picker; emits `select` / `cancel` | ❌ |
| `app-agent-thread.ship` | Turn list; subscribes to `/api/v1/sessions/{id}/events` SSE | ❌ |
| `app-agent-composer.ship` | Input + send + annotation chip strip; POSTs to `/message` with `annotations[]` | ❌ |
| `app-pty-attach.ship` | Wraps `<ship-terminal>` + SSE bytes + `/input` POST + `/resize` POST | ❌ |
| `app-pipeline-board.ship` | Job list + selected-run DAG; polls `/api/v1/jobs` at 2s | ❌ |
| `app-pipeline-step.ship` | Single step card with status dot | ❌ |

All compile (`/_ship/components/<name>.js` returns 200). None has been opened in a real browser this session — no way to do that from here.

## Backend additions

- `ChildHandle::resize(cols, rows)` + `ActorHandle::resize` — `crates/core/runtime/src/supervisor/transport.rs` + `mod.rs`.
- `POST /api/v1/sessions/{id}/resize { cols, rows }` — `apps/shipd/src/sessions/api.rs::session_resize`. Returns 204 on success; 501 for legacy `SessionRegistry` sessions (route through `actor_sessions` to gain it).
- Message body extension: `{ text, annotations: [{type, content, source}] }` — `apps/shipd/src/sessions/api.rs::MessageBody`.
- `render_message_with_annotations()` — provider-agnostic `<context><attachment type="…" source-kind="…" source-ref="…">…</attachment></context>` prefix prepended to prompt. Every `push_mechanism` (Channel/Stdin/Respawn) sees the same shape.
- `studio.chat_turn` payload now carries `text` (annotated), `raw_text` (just user input), `annotations[]` (structured for projection / replay / chip render on reconnect).

## SFC ergonomics — for the LSP/SDK work on the other machine

Issues encountered while authoring the seven SFCs above. Documenting because the LSP / docs / framework iteration is happening elsewhere and these are real:

1. **`{{ props.X }}` Jinja interpolation with empty defaults.** Compiled output shows `<span class="title"> · </span>` where I wrote `{{ props.defaultAgent }} · {{ props.defaultProvider }}`. The format docs claim "rendered once at mount, server-side." Either runtime substitutes per-instance from the host element's actual attributes (and the dehydrated template is what curl returns), or the destructured defaults in `<script>` aren't visible to Jinja, or both. Worth a doc clarification on whether `props.X` is reachable in Jinja and how it resolves per-instance.
2. **No way to programmatically compose components.** Parent SFC mounts a child via embedded HTML tag (`<app-agent-thread data-attr:session-id="$sessionId">`), which means signals propagate as DOM attributes (string-only) and the child's prop decoder picks them up. End-to-end signal-through-DOM hasn't been browser-verified — the child sees the literal Datastar-substituted string as its `props.sessionId`. Worth verifying this works with non-trivial values (booleans, JSON).
3. **No auto-loader for components.** Each component used on a page needs an explicit `<script type="module" src="/_ship/components/<tag>.js">`. The format doc calls auto-load "future SDK work." For now I list five script tags in `apps/shell/index.html`; the pipeline-board view lists two. Manageable but fragile. An LSP / SDK feature that emits the script tags from a manifest of used components would help.
4. **No `data-on:` payload destructuring without inline `event`.** I had to write `data-on:select="onPick(event)"` and have `onPick(e)` read `e.detail`. Convention is fine; just noting that it's not obvious the Datastar `event` global is in scope inside the attribute.
5. **`emit('exit', ...)` from `<app-pty-attach>` triggers a CustomEvent on the host element with bubbles=true.** That's the docs' claim. Haven't verified the bubble actually reaches a `data-on-app:exit__window` listener on a far-away ancestor, but the event should propagate up through the DOM normally.
6. **Style-token availability.** I used a few tokens that may or may not exist (`--ship-color-danger`, `--ship-color-success`, `--ship-color-accent-soft`, `--ship-color-on-accent`, `--ship-radius-pill`, `--ship-radius-lg`, `--ship-shadow-lg`). All have `var(--token, fallback)` fallbacks but the fallback in some cases is just `--ship-color-surface`, which isn't right semantically. Worth grepping `tokens.css` and aligning, or having the LSP catch unknown token references.

## Known gaps — needs your judgment

1. **No browser verification.** Every component compiles; none has been opened in a browser. First task on resume: `just dev-daemon` then visit the shell and `⌘J` to open the modal, navigate to `/pipelines-board` to see the new board. Note any visual / behavior issues.
2. **Per-step `actor_id` wiring on pipeline board.** `<app-pipeline-step>` emits `select` with `stepId`, but the board doesn't know each step's `actor_id` to mount `<app-pty-attach>`. ProcessExecutor sets `actor_id_hint = ctx.slug`, so the per-step actor id is predictable from the slug — but the slug-construction convention varies across executors. To wire output: add `actor_id` to each step's projection record, OR compute `{job_slug}-{step_id}` (verify against ProcessExecutor's actual slug-builder) and try mounting. Skipped here because it needs verification I can't do.
3. **Annotation source wiring (phase 4).** The backend accepts `annotations[]`; the existing `<ship-edit-overlay>` already dispatches `ship:chat:annotate` on `<ship-shell>`. Need to verify the new modal's window-level `ship:chat:annotate` listener catches that event (overlay dispatches on `<ship-shell>`; modal listens on `window` — the DOM event bubbles up so it should reach window). If not, change the dispatch target or the listener target. Then add markdown selection + git diff comment as additional annotation sources — pure UI work in those views.
4. **Old chat impl deletion.** Per the deletion criteria laid out earlier:
    - Spawn-or-attach across all 5 providers ✓ (substrate)
    - Annotation send from at least 3 views ✗ (only edit-overlay wired)
    - Streaming reply renders within 500ms ❓ (needs browser test)
    - Reconnect after modal close → reopen restores thread state ❌ (modal closes session reference; thread re-attaches but doesn't reload prior turns from `/events?from_id=`)
   
   When all four pass, delete `.ship/views/chat/`, `apps/shell/src/chrome/ship-shell.ts`'s right-drawer mounting, and the chat partials.
5. **Right drawer audit + removal.** Today: chat lives in `<aside class="ship-drawer">`. Other panels in the drawer? Audit `<ship-shell>` mounts and `drawer-content`'s consumers before removing. Likely just chat.
6. **Pipeline-board promotion.** Currently lives at `/pipelines-board` alongside the legacy `/pipelines`. When verified equivalent or better, swap routes.
7. **Respawn `push_mechanism`.** Cursor + Gemini default to Respawn (no stdin tested), but the modal calls the legacy `lifecycle::send_agent_message` for that mechanism. Per-turn middleware handles the spawn but the parent placeholder still lives in `SessionRegistry`. Migration path to fold Respawn into `agent_actor` exists in commits but isn't done.

## Browser verification checklist (when you resume)

```
just dev-daemon                             # daemon on :9315 with project SHIP_DIR
just dev                                    # vite on :3100 (or 3100 + offset)
# Open http://localhost:3100/ in browser

# Phase 1 — chat modal
- ⌘J opens the modal
- Picker shows agents + providers
- Select shipwright + claude → modal swaps to thread + composer
- Type "hi", Enter → user bubble appears, assistant reply streams in
- Click title chip → picker re-opens; pick a different agent → new session

# Phase 1.5 — pty attach
- Visit /pipelines-board, select a running pipeline step (if any)
- (or hand-mount <app-pty-attach actor-id="…"> in a test view)
- Type into terminal → bytes echo back
- Resize the panel → verify the child sees new dimensions (e.g., resize during `htop`)

# Phase 3 — annotations
- ⌘⇧E to enter edit-mode
- Click an annotatable element on a view → ship:chat:annotate fires
- Modal opens with chip in composer; chip shows source ref
- Send → backend log shows <context><attachment …> wrapping the prompt

# Phase 5 — pipeline board
- /pipelines-board renders run list
- Selecting a run shows steps with status dots
- Dispatch a job → status updates within ~2s without page reload
- Failed step shows error block in detail pane

# Mesh substrate (already working pre-modal)
- /mesh shows actors (post-merge view rebuild against snapshot endpoint
  isn't in this branch — that's a UI iteration on top of the same data)
- curl /api/v1/mesh/snapshot returns rich actor metadata
- POST /api/v1/actors/{id}/kill-tree stops a tree
```

## Quick recovery if something's broken

- **Daemon won't start (migration error)**: `ship daemon repair-migrations` (and `--mark-applied <version>` if a renamed migration is reported pending).
- **Vite proxy 404 on `/_ship/...`**: confirm `apps/shell/vite.config.ts` has the `/_ship` proxy block (it should after this branch).
- **SFC compiles but doesn't render**: check the `<script type="module" src="/_ship/components/<tag>.js">` is in `apps/shell/index.html`. Auto-loader is future work; explicit script tags are the current contract.
- **Daemon serves wrong project's components**: the daemon resolves project dir from CWD or `SHIP_DIR` env var. `just dev-daemon` sets `SHIP_DIR=$REPO/.ship` explicitly — use it instead of `ship daemon start` when working from a worktree.

## Files at the obvious touch-points if you need to dive

| Concern | File |
|---|---|
| Chat modal logic | `.ship/components/app-agent-modal.ship` |
| Annotation backend | `apps/shipd/src/sessions/api.rs::render_message_with_annotations` |
| PTY resize | `crates/core/runtime/src/supervisor/transport.rs::ChildHandle::resize` |
| Mesh snapshot data shape | `apps/shipd/src/actors_api.rs::build_mesh_snapshot` |
| Provider push routing | `apps/shipd/src/sessions/api.rs::session_message` (PushMechanism match) |
| Per-actor msg_rate | `crates/core/runtime/src/events/kernel_router.rs::RateCounter` |
| Pipeline board polling | `.ship/components/app-pipeline-board.ship::load` |

## Recommended order on resume

1. Browser-verify the chat modal end-to-end. Hit the SFC ergonomic concerns above as you go — many will resolve in 5 minutes once you can see it.
2. Wire per-step `actor_id` so the pipeline board can mount `<app-pty-attach>` per step. This is the next user-facing win.
3. Decide: delete the old `/chat` view + right drawer? Or keep both temporarily.
4. Phase 4 — wire markdown selection + git diff comment as annotation sources.
5. Promote `/pipelines-board` to `/pipelines` when it's at parity.

— Done.
