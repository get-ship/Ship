---
group: Getting Started
title: First-Run Walkthrough
description: The canonical first-run path from install through paired Studio. Local-first; cloud pairing optional.
section: guide
order: 4
---

# First-Run Walkthrough

Ship runs locally. The first-run path has four steps; only the first three are required. Cloud pairing through `ship login` is optional — Ship is fully usable without a getship.dev account.

## 1. Install

The recommended installer is a one-liner:

```bash
curl -fsSL https://getship.dev/install.sh | sh
```

The installer:

1. Downloads the `ship` and `shipd` binaries to `~/.ship/bin/` (override with `SHIP_INSTALL_DIR`).
2. Adds `~/.ship/bin` to your shell's PATH.
3. Registers `shipd` with the OS service manager (`launchd` on macOS, `systemd --user` on Linux).
4. Starts the daemon and opens `http://127.0.0.1:9315/setup` in your browser.

Suppress the browser open with `--no-browser` (or `SHIP_SKIP_BROWSER=1`); skip service registration with `--no-service`. Native installers (`.msi`, `.pkg`, `.deb`, `.rpm`) cover the same ground for end users.

Verify:

```bash
ship --version
ship daemon status
```

## 2. Pick a path

After install you can finish setup either through the **browser wizard** (auto-opened by the installer) or through the **CLI**. Both paths converge to the same configured state. Pick one — running both works but is not necessary.

### Path A — Browser wizard

The `/setup` route serves a four-step wizard. It is reachable only when no workspace exists yet; once a workspace is created the route redirects to the Studio shell at `/`.

The wizard:

1. **Welcome** — orients the user; explains Ship runs locally.
2. **Pick a project folder** — create a new project under `~/Documents/Ship/<name>` or open an existing repo. Posts to `/api/projects`, which scaffolds `.ship/` and registers the workspace.
3. **Connect an AI provider** — pick one of `claude`, `gemini`, `codex`, `opencode`, paste an API key, and test the connection. Saved to `~/.ship/library/settings.jsonc` via `/api/settings`. Skippable.
4. **Done** — links to docs, community, store; "Open Ship" navigates to the dashboard and writes `setup.completed = true`.

Provider selection happens **once** in this path. After completion the wizard is unreachable; the dashboard and settings view are the only places to change provider preferences.

### Path B — CLI

```bash
cd your-project
ship init --provider claude
```

`ship init`:

- Scaffolds `.ship/` with a default `dev` agent.
- Auto-detects existing provider configs (`CLAUDE.md`, `.mcp.json`, etc.) and imports them.
- Writes the chosen provider into `.ship/ship.jsonc` under `project.providers`. Defaults to `claude` if `--provider` is omitted and nothing is detected.
- Activates the `dev` agent and compiles provider-native config files.

When the user opens the browser after `ship init`, the daemon redirects `/setup` to `/shell/` because a workspace now exists, so the wizard does not re-prompt for provider selection.

## 3. Open Studio (optional)

```bash
ship studio --open
```

Studio is the visual interface — the same UI served at `http://127.0.0.1:9315/shell/`. Use it to browse the registry, configure agents, set skill variables, and watch live runtime events.

## 4. Cloud pairing (optional)

Ship does not require a getship.dev account. Cloud is a set of adjacent services — registry publishing, share links, hosted triggers — not a hosted daemon. The local daemon is the source of truth either way.

To pair with cloud services:

```bash
ship login
```

This opens the browser to `getship.dev`, runs PKCE OAuth, and stores a token locally. After pairing you can `ship publish` to share packages and access account-scoped registry features. `ship logout` removes the token; `ship whoami` prints the current identity.

## What gets created

After a successful first run:

| Path | Owner | Purpose |
|------|-------|---------|
| `~/.ship/bin/` | install.sh | `ship`, `shipd` binaries |
| `~/.ship/library/settings.jsonc` | wizard / CLI | Provider preferences, API keys, identity |
| `~/.ship/data.db` | daemon | Workspaces, sessions, events, projections |
| `~/.ship/shipd.pid`, `shipd.port` | daemon | Lifecycle bookkeeping |
| `~/.ship/daemon.token` | daemon | Auth token for paired CLI/MCP clients |
| `<project>/.ship/` | `ship init` | Project manifest, agents, skills |

## Troubleshooting

**Browser did not open.** The installer falls back silently when no opener is available. Visit `http://127.0.0.1:9315/setup` manually, or check the log: `ship daemon status` reports the bind address.

**Daemon already running on a different port.** `ship daemon status` reports the port from `~/.ship/shipd.port`. Stop with `ship daemon stop`, restart with `ship daemon start`.

**Provider step fails the connection test.** The wizard hits `/api/providers/test`. The badge shows the HTTP status; the most common cause is a missing or misnamed API key. You can skip the step and configure later from settings.

## See also

- [Installation](./installation) — install options and platform notes.
- [Your First Project](./first-project) — what `ship init` does in detail.
- [Daemon lifecycle](/reference/cli/commands/#daemon-lifecycle) — `ship daemon start|stop|status|restart`.
- [Studio setup view](/reference/views/setup-view/) — wizard screens and signal flow.
