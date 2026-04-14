# Provider Compat Audit — Pipelines beyond Claude

> Scope: what must change for Ship pipelines (`tdd.yaml`, `smoke.yaml`, etc.)
> to run under Codex, Cursor, Gemini, and OpenCode — not just Claude.
> Research + design. No code changes.

## Executive summary

- **The compiler is multi-provider; the runner is not.** `ship use <provider>`
  emits valid `.claude/`, `.codex/`, `.gemini/`, `.cursor/`, or `opencode.json`
  artifacts today, but `ProcessExecutor` hard-codes `claude` as the spawned
  binary (`apps/shipd/src/supervisor/executors/process.rs:66`) and uses
  Claude-only flags (`--dangerously-skip-permissions`,
  `--dangerously-load-development-channels`, `-p`) at lines 55–64.
- **Pipeline step advancement depends on the Claude Stop hook.**
  `step_monitor.rs:172` waits on `StepActivity::TurnEnded` which is sent only
  by `POST /api/runtime/jobs/end` from `ship hook session-end`
  (`apps/shipd/src/runtime_api.rs:495-505`). Only Claude's compiled settings
  contain an HTTP variant of this hook
  (`crates/core/compiler/src/compile/claude.rs:175-192`); other providers fall
  back to the `ship hook` command path which works but is untested end-to-end.
- **The `ProviderAdapter` trait exists but has only two implementations.**
  `adapter_for()` silently defaults unknown providers to Claude
  (`crates/core/runtime/src/services/provider_adapter.rs:49-54`). Gemini,
  Cursor, OpenCode adapters do not exist.
- **Autostart is a tmux keystroke** — `drive_agent_to_ready` polls for
  Claude's "bypass permissions on" footer
  (`crates/core/runtime/src/services/providers/claude.rs:85`) and sends
  `"Read .artifacts/job-spec.md..."` into the pane. Codex uses `"$ "` +
  `"go"` (`providers/codex.rs:29-30`); neither pattern matches Gemini / Cursor
  / OpenCode TUIs.
- **Recommendation: promote Codex to runtime parity first.** It already has a
  skeleton adapter, compiled TOML config, hooks.json, and agents dir. Gap is
  `ProcessExecutor` provider-dispatch + one-shot prompt flag parity + Stop-hook
  wiring verification. ~3 focused jobs; see §6.

---

## 1. Provider matrix

| Feature | Claude | Codex | Cursor | Gemini | OpenCode |
|---|---|---|---|---|---|
| CLI binary | `claude` | `codex` | `cursor-agent` (docs: [cursor.com/docs/cli](https://cursor.com/docs/cli/overview)) | `gemini` | `opencode` |
| Transport | stdio TUI + headless `-p` | stdio TUI + `exec` non-interactive ([openai/codex](https://github.com/openai/codex)) | stdio TUI + `cursor-agent exec` ([docs](https://cursor.com/docs/cli/reference/exec)) | stdio TUI + `-p` ([gemini-cli](https://github.com/google-gemini/gemini-cli)) | stdio TUI + `opencode run` ([opencode.ai/docs/run](https://opencode.ai/docs/run/)) |
| Hooks | HTTP + shell-command; `.claude/settings.json` `hooks` key | shell-command only; `.codex/hooks.json` ([developers.openai.com/codex](https://developers.openai.com/codex/cli/hooks)) | shell-command; `.cursor/hooks.json` ([cursor.com/docs/agent/hooks](https://cursor.com/docs/agent/hooks)) | shell-command; `.gemini/settings.json` `hooks` ([geminicli.com/docs/hooks](https://geminicli.com/docs/hooks)) | JS plugin file; `hooks.file: "…js"` in `opencode.json` ([opencode.ai/docs/plugins](https://opencode.ai/docs/plugins/)) |
| Permissions model | allow/deny/ask + `--permission-mode` + `--dangerously-skip-permissions` | `approval_policy` in `config.toml` (`never`/`on-failure`/`on-request`) + `--dangerously-bypass-approvals-and-sandbox` | `.cursor/cli.json` permissions object + `--force` | `.gemini/policies/*.toml` + `--yolo` | `permission` block in `opencode.json` + `opencode run --yolo` (effective) |
| MCP config | `.mcp.json` (`mcpServers`) | `.codex/config.toml` `[mcp_servers.*]` | `.cursor/mcp.json` (`mcpServers`) | `.gemini/settings.json` (`mcpServers`) | `opencode.json` (`mcp`) |
| Autonomous flag | `--dangerously-skip-permissions` | `--dangerously-bypass-approvals-and-sandbox` / `--ask-for-approval=never --full-auto` | `--force` | `--yolo` | `--yolo` (docs call it equivalent) |
| Autostart prompt | `-p "<prompt>"` + `--print` | `codex exec "<prompt>"` | `cursor-agent exec "<prompt>"` | `gemini -p "<prompt>"` | `opencode run "<prompt>"` |
| Compiled outputs today | `CLAUDE.md`, `.mcp.json`, `.claude/settings.json`, `.claude/agents/*.md`, `.claude/skills/*` | `AGENTS.md`, `.codex/config.toml`, `.codex/hooks.json`, `.codex/agents/*.toml`, `.agents/skills/*` | `.cursor/mcp.json`, `.cursor/hooks.json`, `.cursor/cli.json`, `.cursor/environment.json`, `.cursor/rules/*.mdc`, `.cursor/agents/*.md`, `.cursor/skills/*` | `GEMINI.md`, `.gemini/settings.json` (merged mcp+hooks), `.gemini/policies/ship.toml`, `.gemini/agents/*.md`, `.agents/skills/*` | `AGENTS.md`, `opencode.json` (mcp + hooks.file + agent), `.opencode/skills/*` |

Compiled-output source: `apps/ship-cli/src/compile/output.rs:9-134` and
`crates/core/compiler/src/compile/{claude,codex,cursor,gemini,opencode}.rs`.

---

## 2. Pipeline-runner assumptions that are Claude-specific

| # | Assumption | File:line | What breaks non-Claude |
|---|---|---|---|
| A1 | Binary is literally `claude` | `apps/shipd/src/supervisor/executors/process.rs:66` | Codex/Cursor/Gemini/OpenCode never spawn |
| A2 | `--dangerously-skip-permissions` flag | `process.rs:57` | Rejected by every other CLI |
| A3 | `--dangerously-load-development-channels server:ship` flag | `process.rs:58` | Claude-proprietary — ignored at best |
| A4 | `--print -p <prompt>` headless convention | `process.rs:59-61` | Codex uses `exec`, Gemini `-p` w/o `--print`, Cursor `exec`, OpenCode `run` |
| A5 | Autostart prompt string is hardcoded for agent re-entry | `process.rs:55`, `providers/claude.rs:86-88` | Non-Claude TUIs don't read stdin the same way |
| A6 | `adapter_for(None/unknown) = ClaudeAdapter` | `runtime/src/services/provider_adapter.rs:49-54` | Gemini/Cursor/OpenCode silently run as Claude |
| A7 | `TmuxExecutor` ready-pattern = "bypass permissions on" | `providers/claude.rs:85`, `executors/tmux.rs:198-234` | Other TUIs never trip `drive_agent_to_ready` → autostart never sent |
| A8 | Channels-menu dismissal: `MENU_TEXT = "I am using this for local development"` | `executors/tmux.rs:200` | Only Claude shows this menu; dead code elsewhere |
| A9 | Step advance driven by `StepActivity::TurnEnded` from Stop hook | `runtime/src/services/orchestrator/step_monitor.rs:170-191`, `runtime_api.rs:495-505` | Non-Claude Stop hooks route through `ship hook session-end` → `signal_job_completed` (`ship-cli/src/hook.rs:211-236`), which *is* wired but untested per-provider and assumes `SHIP_MESH_ID` env propagates |
| A10 | HTTP hook URLs only emitted for Claude | `compiler/src/compile/claude.rs:175-208` | Codex/Cursor/Gemini always round-trip through `ship hook` subprocess → extra latency per tool call |
| A11 | Dispatcher provider auto-detect is ordered-file sniff | `.ship/skills/spawn-agent/scripts/dispatch.sh:143-155` | First match wins; worktree carrying both `CLAUDE.md` and `AGENTS.md` always picks claude |
| A12 | Only `claude` + `codex` get launch flags in dispatch.sh | `dispatch.sh:213-217` | Cursor/Gemini/OpenCode launch with no autonomous/yolo flag |
| A13 | `provider_cli()` whitelists only `claude`/`claude-code`/`codex` | `apps/shipd/src/supervisor/helpers.rs:224` | `send_agent_command` no-ops for the other three |
| A14 | `detect_provider()` in hook CLI only branches on Codex/Gemini/Cursor env vars | `apps/ship-cli/src/hook.rs:22-38` | OpenCode not detected → normalized payload says `"claude"` |
| A15 | Stop hook wiring for Codex/Cursor/Gemini/OpenCode uses generic `ship hook session-end`; path coverage in tests missing | `compiler/src/compile/codex.rs:218-240`, `cursor.rs:65-82`, `gemini.rs:12-20`, `opencode.rs:237-241` | Any one of these misfiring hangs the pipeline at settle timeout |
| A16 | OpenCode hooks reference `contrib/opencode-ship-hooks.js` which is not emitted as a skill file | `opencode.rs:237-241` (no corresponding file write in `output.rs`) | OpenCode pipelines never fire Stop → step never advances |

---

## 3. Gap matrix

Legend: ✅ works · ❌ broken today · 🚫 impossible without new feature code

| Assumption | Claude | Codex | Cursor | Gemini | OpenCode |
|---|---|---|---|---|---|
| A1 Binary spawn | ✅ | ❌ | ❌ | ❌ | ❌ |
| A2 `--dangerously-skip-permissions` | ✅ | ❌ | ❌ | ❌ | ❌ |
| A3 Dev-channels flag | ✅ | ❌ | ❌ | ❌ | ❌ |
| A4 `-p` prompt flag | ✅ | ❌ | ❌ | partial (flag name matches, no `--print`) | ❌ |
| A5 Autostart prompt | ✅ | ❌ | ❌ | ❌ | ❌ |
| A6 Adapter registered | ✅ | ✅ (skeleton) | 🚫 | 🚫 | 🚫 |
| A7 Ready pattern | ✅ | partial (`"$ "` is lossy) | 🚫 | 🚫 | 🚫 |
| A9 Stop→TurnEnded | ✅ | ❌ (untested) | ❌ (untested) | ❌ (untested) | ❌ (hook file missing) |
| A10 HTTP hook | ✅ | 🚫 | 🚫 | 🚫 | 🚫 |
| A12 Dispatch launch flags | ✅ | ✅ | ❌ | ❌ | ❌ |
| A16 OpenCode hook plugin | n/a | n/a | n/a | n/a | ❌ |

Counted "broken today" cells: well over 3 (≈30). Gaps are real and stacked.

---

## 4. Minimum viable Codex parity for `tdd.yaml` + `smoke.yaml`

`smoke.yaml` is pure shell → provider-agnostic already. It exercises dispatch
and git-evidence gathering, not LLM behaviour. Green-light criterion: a
Codex-configured agent worktree still dispatches, commits, and advances steps.

`tdd.yaml` needs a real provider to generate tests and implementation.

Minimum viable change set (Codex first):

1. **Route `ProcessExecutor` through `ProviderAdapter`.**
   Replace the hardcoded `"claude"` command/arg list at
   `process.rs:56-73` with
   `adapter.build_headless(&ctx.slug, &ctx.launch)` and spawn
   `cmd.bin` / `cmd.args`. Extend `CodexAdapter::build_headless` to emit
   `codex exec "<prompt>" --dangerously-bypass-approvals-and-sandbox`.
   (~40 LOC across two files.)
2. **Pass provider down to the process executor.**
   `JobContext.launch.provider` is already threaded to `TmuxExecutor`
   (`executors/tmux.rs:38`); mirror that call in `ProcessExecutor::spawn`.
3. **Verify `ship hook session-end` → `/api/runtime/jobs/end` in Codex.**
   Codex hooks.json (`compile/codex.rs:218-240`) already fires
   `ship hook session-end`. Add an integration test that runs
   `smoke.yaml` under a fake "codex" binary stub and asserts
   `StepActivity::TurnEnded` arrives. (~1 test file.)
4. **Dispatcher launch flags for codex must include autonomous bypass.**
   `dispatch.sh:215-217` currently passes only `MODEL_FLAG` to codex. Append
   `--dangerously-bypass-approvals-and-sandbox` (or `--full-auto`) by default.
5. **Dispatcher provider auto-detect should prefer explicit `ship.provider`
   config over file sniffing** (`dispatch.sh:143-155`). Quick: read
   `.artifacts/agent.json` (compiler emits this) for the canonical provider.

That's it for `smoke.yaml` green. For `tdd.yaml` green, additionally
confirm the Codex `AGENTS.md` render includes the runtime contract section
and the Stop hook reliably fires after each `codex exec` invocation (Codex
currently doesn't fire `SessionEnd` on single-shot `exec` — confirm against
`developers.openai.com/codex/cli/hooks`).

---

## 5. Provider-specific gotchas

| Provider | Gotcha |
|---|---|
| **Claude** | `stream-json` requires `--verbose` or the CLI silently exits 0 — noted inline in `process.rs:50-54`. `Stop` hook fires *every* response turn, not at session end; that's why `step_monitor` settles rather than advances-on-first-signal. |
| **Codex** | `codex exec` is one-shot; `codex` (interactive) is long-lived. Hooks in `.codex/hooks.json` only fire inside the interactive TUI on at least some versions — confirm `SessionEnd` firing in `exec` mode. Config is TOML, not JSON; `output.rs:34-36` specifically skips the JSON merge path for Codex. `$ ` as ready pattern (`providers/codex.rs:29`) is a bash-prompt guess, not a Codex-specific sentinel — unreliable. |
| **Cursor** | Two CLI surfaces: `cursor` (IDE) and `cursor-agent` (headless). Ship only makes sense with `cursor-agent`. Cursor splits tool hooks into `beforeMCPExecution` / `beforeShellExecution` (`compile/cursor.rs:14-22`); `ship hook` normalises them, but payload shape differs from Claude's — `hook.rs:41-92` normalises reasonably but doesn't cover cursor-specific fields (e.g. `toolCallId`). Resume semantics undocumented; assume no session resume. |
| **Gemini** | Settings merge order is finicky — `.gemini/settings.json` receives *both* `mcpServers` and `hooks` in one write (`output.rs:28-32`). A pre-existing user settings file is merged in, but deep conflicts (e.g. user's existing `hooks.SessionEnd`) silently stack: our inject appends to the array rather than replaces, which is desired — but untested with a real pre-populated file. Quit via `/exit` unconfirmed; `exit_command()` not implemented in a Gemini adapter yet. |
| **OpenCode** | Hooks are a *JS plugin file* (`opencode.json` → `"hooks": { "file": "contrib/opencode-ship-hooks.js" }`) — the plugin file is **not emitted** by the compiler (`opencode.rs:237-241`), so no Stop hook fires, so pipelines hang at settle timeout. `--yolo` is effective but not a stable flag on some versions; `opencode run` autonomous mode is the supported path. Session state stored under `~/.local/share/opencode/` — no per-project resume semantics that interact with worktrees. |

---

## 6. Recommended implementation order

**Order:** Codex → Cursor → Gemini → OpenCode.

**Why Codex first:**
- Skeleton adapter exists (`providers/codex.rs`, 39 LOC) — shortest path to
  parity.
- Compiled artifacts (`AGENTS.md`, `.codex/config.toml`,
  `.codex/hooks.json`) are real and tested (`compile/codex.rs`).
- `dispatch.sh` already has a codex branch; only flags need bumping.
- Permissions model (TOML `approval_policy`) is already modelled in
  `compile/codex.rs`.

**Order rationale for the rest:** Cursor next because hooks + permissions are
JSON and already emit; Gemini third because settings.json merge is the
trickiest but hook semantics align with Claude; OpenCode last because the JS
plugin file is not yet generated at all and requires new compiler code.

**Shipping story:** each provider lands behind its own job; `smoke.yaml`
green before `tdd.yaml` green; adapter landing separate from
dispatch-script changes. No omnibus merges.

### Follow-up job stubs (≤ 5 lines each)

1. **`job/runtime-process-executor-provider-dispatch`**
   Replace hardcoded `claude` in `process.rs:55-73` with `adapter.build_headless()`.
   Thread `JobContext.launch.provider` into `ProcessExecutor::spawn`.
   Acceptance: `SHIP_EXECUTOR=process SHIP_PROVIDER=codex` runs `smoke.yaml` end-to-end.

2. **`job/codex-adapter-headless-exec`**
   Flesh out `CodexAdapter::build_headless` to emit
   `codex exec "<prompt>" --dangerously-bypass-approvals-and-sandbox`.
   Add ready-pattern + exit-command that work against `codex exec`.
   Acceptance: integration test in `runtime/src/services/providers/tests_codex.rs` passes.

3. **`job/hook-stop-signal-e2e-per-provider`**
   Add `tests_pipeline_reliability.rs` cases that stub each provider and assert
   `StepActivity::TurnEnded` reaches `step_monitor`.
   Acceptance: 4 new tests (codex, cursor, gemini, opencode) green.

4. **`job/opencode-hooks-plugin-emit`**
   Compiler emits `contrib/opencode-ship-hooks.js` when provider=opencode.
   Plugin POSTs to `/api/hooks/session-end` on turn end.
   Acceptance: `opencode.json` references a file that exists on disk after `ship use opencode`.

5. **`job/dispatch-script-provider-flags`**
   `dispatch.sh:207-217` adds autonomous flags for cursor (`--force`),
   gemini (`--yolo`), opencode (`--yolo`).
   Replace file-sniff provider detection with `agent.json` read.
   Acceptance: dry-run shows correct launch command for each provider.

6. **`job/provider-adapter-registrations`**
   Add `GeminiAdapter`, `CursorAdapter`, `OpenCodeAdapter` to
   `runtime/src/services/providers/mod.rs` + `adapter_for()`.
   Make `adapter_for(None)` return `Err` instead of silently picking Claude.
   Acceptance: unknown provider refuses to spawn with a clear error.

7. **`job/hook-detect-opencode`**
   Extend `detect_provider()` in `ship-cli/src/hook.rs:22-38` to recognise
   OpenCode via `OPENCODE_SESSION_ID` (or equivalent env from
   `opencode.ai/docs/run/`). Normalise its stdin payload.
   Acceptance: snapshot test for OpenCode hook payload.

---

## Appendix: file:line reference index

- `apps/shipd/src/supervisor/executors/process.rs:55-73` — hardcoded claude launch
- `apps/shipd/src/supervisor/executors/tmux.rs:38, 66-74, 198-234` — tmux drives Claude-style ready pattern
- `apps/shipd/src/supervisor/helpers.rs:218-237` — `provider_cli()` whitelist
- `apps/shipd/src/supervisor/job_dispatch.rs:46-53` — executor selection via `SHIP_EXECUTOR`
- `apps/shipd/src/hooks_api.rs:60-133` — unified hook endpoints
- `apps/shipd/src/runtime_api.rs:449-510` — end_job routes Stop → TurnEnded
- `apps/ship-cli/src/hook.rs:22-38, 41-92, 211-236` — provider detect, normalise, signal
- `apps/ship-cli/src/compile/output.rs:9-134` — per-provider file emission
- `crates/core/runtime/src/services/provider_adapter.rs:22-54` — trait + factory
- `crates/core/runtime/src/services/providers/claude.rs:12-88` — ClaudeAdapter
- `crates/core/runtime/src/services/providers/codex.rs:11-35` — CodexAdapter (skeleton)
- `crates/core/runtime/src/services/orchestrator/step_monitor.rs:140-197` — exit+activity select loop
- `crates/core/runtime/src/services/orchestrator/mod.rs:39-62` — `StepActivity::TurnEnded`
- `crates/core/compiler/src/compile/provider.rs:146-225` — static PROVIDERS table
- `crates/core/compiler/src/compile/claude.rs:164-208` — `inject_ship_lifecycle_hooks` (HTTP)
- `crates/core/compiler/src/compile/codex.rs:218-240` — codex hooks.json
- `crates/core/compiler/src/compile/cursor.rs:14-82` — cursor hook mapping + inject
- `crates/core/compiler/src/compile/gemini.rs:10-20, 42+` — gemini hook mapping
- `crates/core/compiler/src/compile/opencode.rs:210-241` — opencode config + hooks.file (plugin NOT emitted)
- `.ship/skills/spawn-agent/scripts/dispatch.sh:139-217` — provider detect + launch flags
