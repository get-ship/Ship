# tests/agents/

End-to-end test suite for the agents domain. Drives the agents-arc by
defining what "working" means as a runnable contract — each slice
either passes (the behavior works) or fails (the behavior is the next
job).

Distinct from `tests/e2e/`
(Playwright UI tests). This suite is **TypeScript + Bun**, no
transpilation, no separate runner — `bun test tests/agents/` discovers
and runs everything.

## Slices

| Path | What it asserts | Status |
|---|---|---|
| `harness-compat/spawn.test.ts` | Each harness in the tier list dispatches → spawns → registers as a kernel actor → cleans up. | ← first slice |
| `harness-compat/events.test.ts` | Each harness emits the lifecycle events Ship expects (session_started, turn_complete, etc.). | planned |
| `harness-compat/hooks.test.ts` | Each harness's hook mechanism fires turn / tool-use / session-end callbacks. | planned |
| `harness-compat/pty.test.ts` | WebSocket attach (`/api/v1/agent_sessions/{slug}/attach`) round-trip per PTY-backed harness. | planned |
| `ui-endpoints.test.ts` | Daemon HTTP routes the agent UI calls (`/api/v1/mesh`, `/actors/{id}`, `/sessions/{id}/message`, etc.). Shape-validates responses without a browser. | planned |
| `annotations.test.ts` | `POST /sessions/{id}/message` with annotations renders to `<context>` XML wrapper and reaches a running agent. | planned |

`harness-compat/` is **nested** under the umbrella because the harness
conformance matrix is a sub-concern; non-compat slices (UI endpoints,
annotations) live as siblings.

## Harness tiers

Defined in `harness-compat/_registry.ts`. Editing the registry
propagates across every slice automatically.

| Tier | Harnesses | Bar |
|---|---|---|
| **T1** | codex, claude, opencode | Must pass when binary is on PATH. CI red on any failure. |
| **T2** | gemini | Should pass; `T2-FLAKY` warning if it breaks, doesn't red CI. |
| **T3** | cursor | Evaluate-as-we-go. Blockers escalate to the human, not the agent. |

Future harnesses (hermes, pi, amp, …) add one entry to `_registry.ts`.

## Running

```bash
# All slices (assumes `just dev` has the worktree daemon running)
bun test tests/agents/

# One slice
bun test tests/agents/harness-compat/spawn.test.ts

# Filter by name
bun test tests/agents/ --test-name-pattern="claude"

# Pretend no binary is installed (useful for CI shape testing)
SKIP_HARNESSES=codex,claude,opencode,gemini,cursor bun test tests/agents/

# Override daemon URL
SHIPD_URL=http://localhost:9315 bun test tests/agents/
```

The suite auto-discovers the per-worktree daemon by hashing the git
toplevel path (matches `scripts/dev/_lib.sh::daemon_dir`) and reading
`port` + `token` from `~/.ship/machine/daemons/<hash>/`. Override with
`SHIPD_URL` + `TOKEN` env if you need a different daemon. Tests skip
individual harnesses cleanly when the binary isn't on `PATH` — CI
passes even on a minimal image.

## Authoring rules

- **Pass when the behavior works; skip when the harness binary is
  missing.** Never fail because of missing tooling — only because of
  broken behavior.
- **Parameterize over the registry.** New slice = one file that
  iterates `HARNESSES`. Don't write per-harness duplicates.
- **Idempotent cleanup.** Every test that dispatches must DELETE its
  session in a `finally` block. The suite's drop-guard catches
  panics, but be explicit.
- **No model API calls intended.** We test the harness CLI tool
  spawn / event / hook contracts, not the AI behind it. If a harness
  phones home during init, accept the cost — don't fight it with
  env-var tricks.
- **300-line cap per file.** Helpers go in `_helpers.ts`; registry data
  in `_registry.ts`. Each `.test.ts` should be the assertions only.

## Why this suite

The agents-arc goal is "agents working end to end" across five (six,
maybe more) harnesses. Today there's no contract test that proves a
harness binary actually launches under Ship's PTY + kernel + mesh
infra. This suite is that contract. Each themed bundle that follows
(push parity, hook analytics, UI work) lands by making more of this
suite go green.
