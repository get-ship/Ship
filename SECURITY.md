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

**Status: filed as task #75.**
**Universal across harnesses that support hooks (Claude Code, future others).**

Ship ships a PreToolUse hook script that the agent compiler emits into
every agent's harness config. The hook receives the tool call JSON at
runtime and runs Ship-aware checks before the call executes. Unlike
Layer 1, the hook sees actual arguments rather than patterns, so it can
catch dynamically-constructed bypasses:

- Any write to `~/.ship/machine/`, `~/.ship/data/`, `~/.ship/projections/`
  regardless of the Bash command's surface shape
- `tar`/`cp`/`mv`/`dd` whose targets include Ship-owned paths
- Direct daemon HTTP from inside the agent process (`curl localhost:<port>`)
- Future: rate limiting, anomaly detection, audit logging

The hook is the **mission-critical layer for Ship integrity** — it is the
defense that holds even when the harness permission list is misconfigured
or compromised.

**What this defends:** runtime intent. An agent attempting to corrupt
Ship state is blocked regardless of how the command is phrased.

**What this does NOT defend:**

- Harnesses that do not support hooks at all (none currently in scope
  but worth tracking)
- An agent that disables hooks via harness config — the floor prevents
  the obvious cases (write to `.claude/settings.json` is protected),
  but a sufficiently determined agent could try harder

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
