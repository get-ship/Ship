---
group: Schema Reference
title: .ship/session.json
section: reference
order: 8
---

# `.ship/session.json`

Per-worktree record of the active harness session, used by the dispatch
endpoint to wire `resume: true` to the right harness-native CLI flags.

Written by the runtime the first time the harness emits a `SessionStarted`
event in a worktree. Overwritten on each subsequent launch
(last-session-wins). Read by `POST /api/v1/agent_sessions/dispatch` when
`resume: true` to recover the session id and validate harness parity.

## Path

```
<worktree>/.ship/session.json
```

## Schema

```jsonc
{
  "harness": "claude",                 // string — harness id ("claude", "codex", "opencode", ...)
  "session_id": "sess-abc-def-123",    // string — harness-native session id
  "agent": "rust-lane",                // string — agent profile id that opened the session
  "started_at": "2026-05-18T18:33:01Z" // string — RFC3339 UTC timestamp captured at write
}
```

All four fields are required. The file is written atomically by the
runtime — partial files indicate a crashed write, not a supported state.

## Lifecycle

1. **Created** by the runtime once the harness's first JSON-stream
   event includes its session id (Claude's `system.init.session_id`,
   Codex's `thread.started.thread_id`, etc.).
2. **Read** by the dispatch endpoint on every `resume: true` request.
   Missing file → `400 BAD_REQUEST` ("no `.ship/session.json` in
   worktree …").
3. **Validated** against the requested harness. Mismatch → `400` ("harness
   mismatch — request asked for `X` but the session was opened with
   `Y`"). Resume is never silently retargeted across harnesses.
4. **Overwritten** on each new launch — there is exactly one active
   session per worktree at a time.

## Per-harness coverage

| Harness  | `session_resume` cap | Writes `session.json` | Resume CLI                          |
|----------|----------------------|-----------------------|-------------------------------------|
| claude   | yes                  | yes (from `system.init`) | `claude --resume <uuid> --fork-session` |
| codex    | yes                  | yes (from `thread.started`) | `codex exec resume <id>`             |
| opencode | yes                  | not yet emitted by adapter | `opencode run --session <id>`        |
| gemini   | no                   | no                    | resume not supported — returns `400` |
| cursor   | no                   | no                    | resume not supported — returns `400` |

OpenCode supports resume at the protocol level but its CLI stream parser
does not currently surface a `SessionStarted` event from `opencode run`
output, so the file is only present today for harnesses whose adapter
emits the event. Once the OpenCode SSE rewrite lands, the file will be
populated for OpenCode runs too without further changes to dispatch.
