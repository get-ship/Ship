# Sidecar parity audit

Every Ship feature that an agent uses must work the same whether the
agent runs on the host (Tier 0) or inside a sandbox (Tier 1-4). This
doc enumerates the surfaces, with current status, gap, and plan.

Status legend:

- **direct-mount** — host provides this via a bind mount or env var;
  no sidecar work needed.
- **sidecar-endpoint** — sidecar has (or needs) an HTTP/SSE endpoint
  for it.
- **gap** — neither works today.

## 1. Harness CLI execution

The orchestrator spawns a harness binary (claude / codex / opencode /
gemini / cursor-agent). Each binary needs its install path, its config
dir, an MCP config, optional model selection, env vars, working
directory, stdio, exit code.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Spawn the binary | `ProcessExecutor::spawn` builds a `tokio::Command` | sidecar `/v1/exec` does same inside container | sidecar-endpoint (works for claude — `run_claude_sidecar` example) |
| stdin / stdout / stderr | piped to ActorHandle | sidecar `/v1/events` SSE relays stdout/stderr; `/v1/stdin` accepts input | sidecar-endpoint (works; PTY upgrade gap below) |
| Exit code + signal | `try_exit`, `stop` on ActorHandle | sidecar emits `SidecarEvent::Exited`; `/v1/signal` for stop | sidecar-endpoint |
| Per-harness env (e.g. `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) | composed by `harnesses::*` modules, set on Command | host composes, passes via `ExecRequest.env`; sidecar plumbs through | sidecar-endpoint (works) |
| Working directory | `Command::current_dir` | `ExecRequest.cwd` | sidecar-endpoint |

**Plan:** none — works today for non-interactive runs. See §6 for PTY.

## 2. MCP servers

Agents call out to MCP servers (`.mcp.json` / `.ship/mcp/*`) over
stdio or HTTP. Stdio servers are subprocesses of the agent; HTTP
servers are network endpoints.

| MCP server kind | Host | In sandbox | Status |
|---|---|---|---|
| **stdio** (subprocess) — `node`, `bun`, `python`, etc. | spawned by the harness binary as its own child | the harness inside the container spawns it; works as long as the runtime (`node`, `bun`, `python`) is in the rootfs | direct-mount, conditional (rootfs needs the runtime) |
| **HTTP** to localhost (developer-run MCP server) | direct TCP to 127.0.0.1 | container reaches host via shared netns (today) or unix-socket bind-mount (production) | **gap** when we move to isolated netns — needs unix-socket bridge |
| **HTTP** to external host | direct TLS | proxy allowlist must include the MCP host | sidecar-endpoint (allowlist) |
| MCP server config (`.mcp.json`) | composed by `config::mcp`, written to harness session dir | sidecar provides the file via bind-mount of session dir (read-only) | direct-mount |

**Plan:**
- Audit rootfs runtimes: today's Ubuntu+bun rootfs has `bun` + `node`
  (via bun); add `python3` to rootfs build for stdio MCP servers that
  need it.
- Unix-socket bridge for host-local MCP: out of scope until the
  shared-netns retirement.
- Allowlist already supports per-agent additions (`isolation.allowlist`
  field in agent profile).

## 3. Credentials

The hardest one. Agents need OAuth tokens / API keys to call out to
Anthropic / OpenAI / OpenRouter / etc. Three approaches:

| Approach | Status | Trade-off |
|---|---|---|
| A — bind-mount host `~/.claude/.credentials.json` | works (`run_claude_sidecar`) | container exploit reads host token |
| B — per-sandbox `EphemeralFile` copy on prepare(), destroy on drop | works (`EphemeralFile`) | exploit still reads token; just contained to lifetime |
| C — sidecar broker via `/v1/secret/get`; shipd holds creds | type defined (`sidecar::protocol::SecretGetRequest`); no impl | production answer; token never lands in container |

**Plan:**
- Implement option C: sidecar emits `secret.requested { name }` event;
  shipd resolves from OS keychain (`account/store.rs`) or the legacy
  `credentials.jsonc`; replies with `secret.granted { name, value }`
  via a one-shot reply channel; sidecar exposes the value as an env
  var or file injected at exec time.
- For Claude Code OAuth specifically: refresh-token flow happens
  *outside* the sandbox; sidecar requests "current access token" and
  shipd handles refresh.
- Capability: `policy.secrets: ["anthropic"]` enumerates which
  keychain entries the agent may request. Default empty; profile
  opts in.

## 4. Hooks

`.ship/hooks.jsonc` declares hook commands that fire on kernel events
(file changes, job lifecycle, manual triggers). Hooks need to run with
the same isolation as the agent (or stronger) — they touch user
workspace and run untrusted user-authored scripts.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Hook event delivery | kernel router → hook engine in shipd | shipd still owns the hook engine (host); commands dispatch out to sidebar | direct (host owns engine) |
| Hook command execution | spawned by hook engine | should route through sidecar `/v1/exec` when the hook is associated with a sandboxed agent | **gap** |
| Hook env access | inherits shipd env | needs explicit allowlist of which env keys pass through | **gap** |

**Plan:**
- Hook engine learns to dispatch via `Isolator` when an agent context
  is in scope; falls back to host spawn for hooks not tied to an
  agent.
- One-shot exec — hooks don't need a long-lived container; route
  through the same sidecar that's already running the agent if any,
  else through a transient `Noop` (host).

## 5. File watching

Agents emit file changes the user sees in the UI (mesh / pipeline).
Today this is a host-side `notify` watcher on the workspace dir.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Watch agent's workspace dir | host `notify` on the bind-mounted dir | identical — host sees container writes via the bind mount | direct-mount |
| Watch container-internal paths (e.g. `/root/.cache`) | n/a | not needed; if it matters we bind-mount and host watches | direct-mount |

**Plan:** none. The bind-mount model means the host watcher Just Works.

## 6. PTY / interactive harness

Some harnesses (`claude` interactive, `codex`, `cursor-agent`) want a
real TTY. Today the sidecar uses plain pipes — the agent emits a
warning ("no stdin data received in 3s") and may fall back to a
non-interactive mode.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Allocate PTY | `ChildHandle` (substrate, via portable-pty) | **gap** — sidecar uses Tokio Command piped stdio | **gap** |
| Resize | `ChildHandle::resize(cols, rows)` | sidecar `/v1/exec` needs an optional `pty: { cols, rows }` and `/v1/resize` | **gap** |
| WS attach from host | `apps/shipd/src/routes/agent_sessions_attach.rs` | route works for host-PTY; needs to delegate to sidecar's PTY when sandboxed | **gap** |

**Plan:**
- Sidecar adds `pty` field to `ExecRequest`; when set, sidecar spawns
  with `portable-pty` (already a workspace dep) instead of
  `tokio::process::Command`.
- Sidecar exposes a `/v1/exec/{id}/resize` endpoint and surfaces
  PTY events through `/v1/events` as before.
- Shipd's attach route detects whether the agent is sandboxed; if so,
  it opens a WS proxy to the sidecar instead of subscribing to the
  local ActorHandle.

## 7. Log progress / structured events

Skills + agents call `log_progress(...)` via MCP. The progress event
travels MCP → harness → ship MCP server → kernel.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| `log_progress` MCP call | local stdio MCP server (ship MCP) | the ship MCP server runs *inside the sandbox*, spawned by the harness as a stdio child; it then needs to reach the kernel | **gap** |
| Kernel reach | direct via `KernelRouter` in shipd | container can't reach shipd's in-process router; needs an HTTP/SSE bridge from the in-container MCP server back to shipd | **gap** |

**Plan:**
- Ship MCP server, when launched in a sandbox, talks to shipd's
  daemon API over HTTP (the same API the CLI uses), reached via the
  proxy on `127.0.0.1:<sidecar-port>` over a unix socket later.
- The agent's keychain (per-agent credential) signs API calls so the
  daemon trusts them. Reuses the same broker as §3.

## 8. Workspace + git

Each agent runs against a worktree prepared by the upstream
`IsolationStrategy` (note: that one prepares the *directory*; this
crate sandboxes the *process*). The worktree is bind-mounted in.

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Worktree files | direct path | bind-mounted at identical path (`/work` or the host path passthrough convention) | direct-mount |
| `git` binary | host PATH | rootfs has `git` (Ubuntu pkg) | direct-mount |
| `gh` binary | host PATH | **gap** — not in rootfs; add when needed | **gap** |
| Git creds (`~/.gitconfig`, `~/.git-credentials`) | direct | per-agent profile may opt in to bind-mount these read-only | direct-mount, opt-in |

**Plan:**
- Add `gh` to rootfs build when first agent needs it (claude-code
  itself sometimes shells out to gh).
- Document the opt-in for git creds in the agent profile schema.

## 9. Shipd HTTP API

Some skills + the in-sandbox MCP server need to call shipd's HTTP API
(register a pipeline run, append session log, fetch a setting).

| Surface | Host | In sandbox | Status |
|---|---|---|---|
| Reach shipd from sandbox | `127.0.0.1:<daemon.port>` | shared netns reaches it (today); production wants unix socket | **gap** when netns isolates |
| Auth | implicit (same user) | needs a per-sandbox token issued by shipd, scoped to the agent's permissions | **gap** |

**Plan:**
- Per-sandbox bearer token, minted by shipd at `prepare()`, injected
  as `SHIP_DAEMON_TOKEN` env. Scope mirrors the agent's
  `permissions_tier`.
- Unix-socket transport when we move off shared netns.

## 10. File caps / size

We're at 462 LOC for `youki_sidecar.rs`. The PTY work below will push
it over. Plan: factor stdio handling into its own module before
adding PTY.

---

## Gap summary

Ordered by leverage (how many features it unblocks):

1. **Credential broker (option C)** — unblocks §3, §7, §9
2. **PTY in sidecar** — unblocks §6, removes the "no stdin" warning
3. **Daemon API access from sandbox** — unblocks §7, §9
4. **Hook dispatch through Isolator** — unblocks §4
5. **Unix-socket transport** (replace shared-netns TCP) — unblocks
   real netns isolation; depends on container topology change
6. **Rootfs additions** (python3, gh) — small one-off as needed

Phasing: 1 → 2 → 3 in that order. 4 onwards parallelizable.
