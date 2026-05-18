# Ship security model

How Ship defends user data and Ship integrity against agent and process-level
threats. This document is the source of record — when defenses change, update
this file in the same commit.

## Threat model

Ship runs on a user's machine alongside agent harnesses (Claude Code,
opencode, Codex, Gemini, Cursor). Agents are powerful — they can read
files, run shell commands, make HTTP requests — and they run as the same
Unix user as the human. That makes the relevant threats:

- **Credential exfiltration.** An agent reads SSH keys / cloud credentials /
  the user's Ship account token and sends them off-machine, where the leak
  enables further compromise without needing access to this machine.
- **Ship state corruption.** An agent modifies Ship's event store or
  projections (`~/.ship/data/`, `~/.ship/projections/`), causing silent
  kernel inconsistency, lost work, or boot failures.
- **Ship account abuse.** An agent uses the user's stored gateway account
  token to publish under their name, change billing, log them out, or
  otherwise act on the user's external identity without consent.
- **Other-process compromise.** A different process running as the same
  user (a script, a build tool, a malicious download) does any of the
  above without involving an agent.

Threats explicitly out of scope at the application layer:

- **Compromised root.** If an attacker has root, Ship's protections are
  irrelevant.
- **Stolen device.** Whole-disk encryption is the user's responsibility.
- **Network-layer attacks** between the user and `api.getship.dev`. TLS
  + Better Auth's cookie handling cover this; not a Ship concern.

## Defense layers

Three layers, listed in order from most universal to strongest. Each layer
adds defense where the layers below it stop short. No layer alone is
sufficient.

### Layer 1 — Harness permissions (the floor)

**Status: in production as of `67b946b7`.**
**Source:** `apps/ship-cli/src/compile/permissions.rs` (`FLOOR_DENIES`)
**Universal across harnesses.**

The agent compiler bakes a deny list into every agent's harness config
(`.claude/settings.json`, `opencode.json`, etc.) regardless of preset,
profile, or per-agent override. The floor cannot be removed by user
configuration; attempts to `allow` a floor-protected pattern do not
override at runtime because every supported harness applies deny
precedence.

The floor categorically denies:

- Reads of cross-machine credentials (SSH keys, cloud-provider creds,
  package-registry tokens, password stores, browser credential DBs)
- Reads of local-only Ship credentials (`~/.ship/machine/**`)
- Writes to Ship state (`~/.ship/data/**`, `~/.ship/projections/**`)
- Reads of shell history (often contains unredacted credentials)
- Common bypass patterns (`cat`/`base64`/`xxd` on protected paths,
  `curl | sh`, `sudo`, `rm` on `~/.ship/`)

**What this defends:** the obvious case. An agent that respects the
harness permission system cannot read credentials or corrupt Ship state
through tool calls matching the floor patterns.

**What this does NOT defend:**

- An agent that bypasses the harness entirely (e.g., `--dangerously-skip-permissions`,
  a malicious bash subprocess spawning Python). The floor lives in the
  harness; it has no effect if the harness isn't consulted.
- A clever Bash pipeline that evades the pattern matcher (e.g., constructing
  paths via concatenation, redirecting through unusual binaries).
- File reads by other processes running as the same user.

### Layer 2 — PreToolUse hooks (Ship integrity)

**Status: in production as of `job/pretooluse-hooks`.**
**Source:** `crates/core/cli-framework/src/hooks/tool_gate.rs` (gate logic),
`apps/ship-cli/src/hook.rs::run_before_tool` (transport).
**Wiring:** `crates/core/compiler/src/compile/{claude,gemini,codex,cursor}.rs::inject_ship_lifecycle_hooks`
already emits `ship hook before-tool` into every supported harness's
PreToolUse equivalent; the gate runs inside that handler — no separate
binary, no parallel install path.
**Universal across harnesses that support hooks (Claude Code, Gemini,
Codex, Cursor today).**

The hook receives the tool-call JSON from the harness on stdin and runs
Ship-aware checks before the call executes. Unlike Layer 1, the hook
sees actual arguments rather than glob patterns, so it catches
dynamically-constructed bypasses Layer 1's literal-glob matcher misses.

**Rules enforced:**

- **Write / Edit / MultiEdit / NotebookEdit** to any path inside
  `~/.ship/machine/`, `~/.ship/data/`, or `~/.ship/projections/` — match
  is segment-aware (`~/.ship/datasets/` is not flagged).
- **Write** inside `.git/` (would let an agent rewrite refs) or
  `.claude/` (would let an agent disable the hooks themselves).
- **Bash** verbs `tar`/`cp`/`mv`/`dd`/`rsync`/`tee`/`chmod`/`chown`/`ln`/
  `install`/`base64`/`xxd`/`od`/`hexdump`/`cat`/`head`/`tail`/`less`/
  `more`/`rm` paired with a Ship-protected path in any argument
  position. `sudo` / `env KEY=v` / `timeout` prefixes are stripped before
  identifying the verb.
- **Bash redirects** (`>`, `>>`, `&>`, `&>>`) whose target token is a
  Ship-protected path, regardless of the leading verb.
- **Bash shell-wrappers** — `bash -c "..."`, `sh -c "..."`, `eval "..."`
  recursively re-enter the gate so a quoted inner script cannot evade
  the verb rules. Bounded to depth 4.
- **Direct daemon HTTP** — `curl`/`wget`/`http`/`httpie`/`xh` targeting
  any of `localhost:<port>`, `127.0.0.1:<port>`, `[::1]:<port>`,
  `0.0.0.0:<port>` where `<port>` is the live daemon port (discovered
  via the same per-worktree / singleton walk the CLI uses). Legitimate
  Ship operations go through the `ship` CLI or MCP tools, which
  authenticate and audit-log.

**Failure mode:** fail-closed. Malformed hook JSON or unparseable shell
denies the call rather than waving it through — the threat model
weighs "false-positive deny" against "smuggled bypass" and decides in
favour of the safer side.

**Protocol:** On deny, the gate writes Claude Code's structured
permission-decision JSON to stdout and exits 0
(`{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"..."}}`).
This is the protocol path that surfaces a human-readable reason to the
model, so the agent can explain to the user why the call was blocked
and suggest a Ship-native alternative. Other harnesses (Gemini, Codex,
Cursor) either consume the same shape or ignore stdout — either way the
downstream daemon roundtrip is skipped because there is no tool call to
log.

**Performance:** The gate is pure logic plus one optional read of the
daemon port file (skipped when no daemon is running). Measured well
under the 50 ms hook budget; integration test runtime including process
spawn is ~0.5 s, which is dominated by process startup, not gate work.

**What this defends:** runtime intent. An agent attempting to corrupt
Ship state, exfiltrate a credential via constructed paths, or call the
daemon directly is blocked regardless of how the command is phrased,
including via `$HOME` concatenation, `bash -c` wrapping, or `env`/`sudo`
prefixing.

**What this does NOT defend:**

- **Harnesses that do not support PreToolUse hooks at all** — none are
  currently in scope, but future additions need an audit before they
  ship.
- **An agent that disables the hook via harness config** — the gate
  blocks writes to `.claude/`, and Layer 1 separately denies
  `.claude/settings.json`, but a sufficiently determined agent that
  combines a Layer-1 bypass with a Layer-2 bypass could try. Closing
  that gap is Layer 3's job.
- **An agent escaping the harness entirely** (`--dangerously-skip-permissions`,
  spawning a Python subprocess that doesn't consult Claude Code at all).
  Hooks live inside the harness; they have no effect if the harness
  isn't invoked. Layer 3 is the only defense against this class.
- **File reads by other processes running as the same Unix user** —
  that's a keychain / Unix-socket-peercred concern, tracked separately
  under "Account token storage" below.
- **Audit logging of denied calls** — filed as a follow-up; for now,
  denials are visible to the model (via the reason string) and to
  whoever inspects the harness's hook transcript, but are not
  persisted to the event store.

### Layer 3 — OS sandbox (future)

**Status: spec dispatched as `local-sandbox-spec`; implementation post-launch.**
**Universal across harnesses; kernel-enforced.**

Local microVM (Apple Vz / Cloud Hypervisor / Hyper-V) or process-level
sandbox (`sandbox-exec`, `bubblewrap+seccomp`) wraps each agent process.
The agent runs in an environment that cannot see `~/.ship/machine/`,
cannot spawn processes outside its sandbox, cannot bind ports outside
its allowed range. This is the only layer that defends against an agent
escaping the harness entirely.

The spec doc is the current authoritative design. Until it lands,
Layer 2 is the strongest defense against intentional misuse and Layer 1
catches the unintentional cases.

## Additional protections, not full layers

### `CallerKind` gate on daemon account routes

**Status: lands with this commit arc.**

Daemon routes under `/api/v1/account/*` enforce that the caller is
`CallerKind::Cli` (daemon-token-authed, indicating user-present
interactive access). MCP session tokens are categorically denied,
regardless of role scope. This is defense-in-depth — it prevents the
obvious accident of an MCP agent with a `contributor`-role session
token unlocking account ops. It is not a defense against an agent that
has already read the daemon token from disk; that case is closed by
Layers 1 + 2 + 3.

### Account token storage

**Status: this commit arc moves token to daemon's own dir; keychain is a
filed follow-up.**

The gateway account token (Better Auth API key) is stored under
`~/.ship/machine/daemon/account.jsonc` (mode 0600), owned by the daemon
process. It is NOT written to the shared `credentials.jsonc` the CLI
historically used. Future: OS keychain (`keyring-rs`, macOS Keychain
with GUI prompt-on-access, Linux Secret Service, Windows Credential
Manager) — filed as a follow-up.

### Watcher path audit

**Status: filed as a follow-up task.**

Ship's file-watching systems (`collect-skill-docs.ts --watch`, the view
hot-reloader, daemon-internal watchers) explicitly exclude
`~/.ship/machine/` from their globs. Audit confirms no accidental glob
into credential paths; regression test prevents reintroduction.

### Cross-machine secrets and gateway hardening

**Status: filed individually.**

- CORS allowlist + Origin header validation + CSP headers on daemon HTTP
- Per-user TLS via account-issued subdomain (`<userid>.local.getship.dev`)
  — design doc captured; implementation post-launch when multi-tenant
  becomes pressing
- Unix domain socket + SO_PEERCRED for daemon HTTP — replaces loopback
  HTTP with per-PID authentication

## Permission floor — exact list

The authoritative source is `FLOOR_DENIES` in
`apps/ship-cli/src/compile/permissions.rs`. The list below is for
reference and may lag the source.

**Cross-machine credentials (Read denied):** `~/.ssh/**`, `~/.gnupg/**`,
`~/.aws/**`, `~/.azure/**`, `~/.gcloud/**`, `~/.config/gcloud/**`,
`~/.kube/config`, `~/.netrc`, `~/.git-credentials`,
`~/.config/git/credentials`, `~/.docker/config.json`,
`~/.cargo/credentials.toml`, `~/.npmrc`, `~/.pypirc`,
`~/.password-store/**`, `~/.config/Bitwarden/**`,
`~/.config/1Password/**`, `~/Library/Keychains/**`,
`~/Library/Cookies/**`, Firefox / Chrome credential DBs.

**Local-only credentials (Read denied):** `~/.ship/machine/**`.

**Ship state (Write/Edit denied):** `~/.ship/data/**`,
`~/.ship/projections/**`.

**History files (Read denied):** `~/.bash_history`, `~/.zsh_history`,
`~/.psql_history`, `~/.mysql_history`, `~/.python_history`.

**Bypass patterns (Bash denied):** `cat`/`head`/`less`/`xxd`/`base64`
on protected paths, `curl | sh` and `wget | sh` variants, `sudo:*`,
`rm` on `~/.ship/` in any form.

## Reporting

Security issues: email `security@getship.dev`. Do not file public
GitHub issues for vulnerabilities.
