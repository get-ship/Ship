---
group: Schema Reference
title: KV Settings
section: reference
order: 11
---

# Ship KV Settings

Ship persists daemon-managed preferences in the `daemon_settings` table of the platform SQLite database (`<ship_home>/machine/db/platform.db`). Settings are simple key → JSON-value pairs, stored alongside the kernel event log so the daemon has ACID guarantees for state that needs to evolve in lockstep with events.

This page documents:

- The four-tier configuration resolution chain Ship uses to pick a value at runtime.
- Every known setting key — type, default, the code path that reads it, and whether it has a project-file equivalent.
- How to read, set, and override settings from the CLI, HTTP API, and code.

## Four-tier resolution chain

When Ship asks "what is X?", it walks four tiers in order and uses the first answer:

```
1. Per-launch override   (CLI flag, or SHIP_* env var when applicable)
2. Project file          (.ship/ship.jsonc — identity-shaped values only)
3. User KV               (daemon_settings table; `ship config set` / settings UI)
4. Compile-time default  (setting_default() in runtime_settings.rs)
```

Each tier exists for a different reason:

- **Per-launch override.** Ephemeral. `shipd --port 9400` or `SHIP_DAEMON_MODE=production shipd` wins for the lifetime of that process and is never persisted. See the [env-vars reference](./env-vars.md) for the full list.
- **Project file.** Identity-shaped values that belong to the project itself live in `.ship/ship.jsonc` so they travel with the repo. These are values that, if they ever diverged between users on the same project, the project would break (e.g. project id). They never get a compile-time default.
- **User KV.** Per-machine preferences the user wants Ship to remember — terminal program, daemon port, watcher tuning. Stored in `daemon_settings`. Read via `ship_platform::config::get_setting(key)`, written via `set_setting` or `set_setting_and_emit`.
- **Compile-time default.** Sensible last-resort values returned by `setting_default()` in `crates/core/ship-platform/src/config/runtime_settings.rs`. Reserved for non-identity infrastructure choices (terminal, port, watcher debounce). Identity-shaped values like "which AI harness" or "which model" **never** get a compile-time default — the user must make an explicit choice on first run, since silently picking an AI they haven't installed or paid for is a footgun (see `runtime_settings.rs:14-19`).

Not every setting uses every tier. The "scope" column below records which tiers actually fire for each key.

## Reading and writing

| Surface | Read | Write |
|---|---|---|
| CLI | `ship config get <key>` | `ship config set <key> <value>` |
| HTTP API | `GET /api/v1/settings/{key}` and `GET /api/v1/settings` | `POST /api/v1/settings` with `{ "key": …, "value": … }` |
| Rust code | `ship_platform::config::get_setting(key)` | `ship_platform::config::set_setting` (silent) or `set_setting_and_emit` (broadcasts `config.changed`) |
| Studio | `views/settings/` page | Same page, hand-built per section. New keys do **not** auto-surface — adding a UI control is a separate step. |

## Known setting keys

The canonical list lives in `setting_default()` and `get_all_settings()` in `crates/core/ship-platform/src/config/runtime_settings.rs`. Adding a key means adding rows to both. The table below mirrors what those functions return today.

| Key | Type | Default | Scope | Read by | Notes |
|---|---|---|---|---|---|
| `terminal.program` | string | `"tmux"` | env > KV > default | `crates/ship-supervisor/src/terminal_launcher.rs:89` (via `SHIP_DEFAULT_TERMINAL`); KV via `get_setting` | One of `tmux`, `kitty`, `wezterm`, `iterm2`, `terminal`. The `SHIP_DEFAULT_TERMINAL` env var overrides the stored value. |
| `daemon.port` | string (u16) | `"9315"` | CLI > worktree-derived > default | `apps/shipd/src/lib.rs:194-197` — `--port` CLI flag wins, otherwise the daemon derives a worktree-specific port from `worktree_port_for`; falls back to 9315. | Stored as a string for symmetry with the other settings; coerced where used. |
| `cmdk.send_key` | string | `"enter"` | KV > default | Studio command bar | `enter` sends on Enter (Shift+Enter for newline); `cmd+enter` inverts the binding. |
| `watcher.debounce_ms` | u64 (ms) | `200` | KV > default | `crates/core/ship-platform/src/config/watcher.rs:71` via `config::resolve_watcher_debounce()` | Debounce window for the declarative `.ship/` watcher (agents, skills, views, pipelines, rules). Read once at daemon startup; changes require a daemon restart. |
| `git.log_limit` | usize | `50` | request param > KV > default | `apps/shipd/src/routes/git_api.rs:224` and `apps/shipd/src/services/registry/git.rs:45` via `config::resolve_git_log_limit()` | Default value for the `limit` field on `GET /api/v1/git/log` and the `git.overview` registry op when the caller omits it. |
| `hooks.max_depth` | u32 | `3` | KV > default | `crates/core/ship-platform/src/hooks/engine.rs` via `config::resolve_hooks_max_depth()` | Maximum hook chain depth. Events that arrive carrying a `_hook_depth` at or above this value are dropped without dispatch. Read on every event, so changes take effect immediately. |

### Forthcoming keys (job/launch-defaults)

Three identity-shaped defaults are specced in `job/launch-defaults` and will land separately. They have no `setting_default()` entry on purpose — they require an explicit choice on first run (see the resolution-chain note above).

| Key | Type | Default | Notes |
|---|---|---|---|
| `harness.default` | string | none | Which AI harness (`claude`, `codex`, `cursor`, `gemini`, `opencode`) Ship uses by default when a job spec omits one. |
| `harness.default_model` | string | none | Model id passed to the default harness. |
| `agent_session.auto_resume` | bool | TBD | Whether Studio auto-resumes a paused agent session. |

Once `job/launch-defaults` merges, this section will move into the main table.

## Adding a new setting

1. Add a row to `setting_default(key)` in `crates/core/ship-platform/src/config/runtime_settings.rs`. Document the type in a sibling comment. If the value is identity-shaped (i.e. would silently pick a third-party service for the user), return `None` and document why.
2. Add the same `(key, default-string)` pair to `get_all_settings()` so the settings UI and `GET /api/v1/settings` list include it.
3. Read the value via `get_setting(key)` (or one of the `resolve_*` helpers) — never read a constant directly at the call site.
4. Add a row to this page in the "Known setting keys" table.
5. Add a test (alongside `resolver_tests` in `runtime_settings.rs`) asserting the resolver returns the historical default with nothing stored and the override when stored.
6. Optionally surface it in `views/settings/page.sui` — the page is hand-built per section, so new keys do not appear automatically.

## Migration notes

Historically the daemon stored these values in the `kv_state` table under the `settings` namespace. `drain_legacy_settings_kv()` runs once per startup to migrate any remaining rows into `daemon_settings` and delete them; the migration is idempotent and a no-op on a clean install.
