---
title: "Ship Home Layout and Scope Convention"
description: "ADR: ~/.ship/ splits into library/ (portable) and machine/ (runtime); canonical customization-vector list applied at project and user scopes."
group: ADRs
order: 20
audience: internal
status: accepted
date: 2026-04-24
supersedes: []
tags: [storage, layout, scope, env, ship-home]
---

# Ship Home Layout and Scope Convention

## Context

`~/.ship/` grew organically around Ship's original focus — agents, skills, MCP, rules. Those four were first-class top-level directories. When collections, views, pipelines, and triggers arrived, they did not get equal treatment:

- `~/.ship/schemas/` holds **collection schemas** (`note.jsonc`, `systems.jsonc`, `task.jsonc`) — a legacy directory name that predates the collections concept.
- There is no user-scope `views/`, `pipelines/`, `triggers/`, `collections/`, or `themes/` — those customization vectors exist only at project scope.
- `permissions` has a project-scope JSONC file but no user-scope equivalent.
- `mcp.jsonc` is a single file at root while other vectors are directories; the shape is inconsistent.
- Runtime state (`kernel/`, `cache/`, `crashes/`, `logs/`, `sessions/`, `data.db`, `platform.db`, `hook.token`, `network.pid`, `archive/`) is mixed with user content (`agents/`, `skills/`, `views/`, `projects/`).
- `config.toml` is the only TOML file in a codebase where every other config uses JSONC. It also mixes portable settings (`[dispatch]`, `[terminal]`) with machine-local state (`[worktrees].dir`).
- Daemon IPC handshake files (`hook.token`, `network.pid`) sit at root, treated as peers of user content.
- Code paths compute `~/.ship/…` ad hoc in 40+ call sites, with no central typed accessor. Environment variables (13+ `SHIP_*` vars) are scattered, undocumented, and referenced by string literal everywhere.
- Dead architecture from the per-actor-DB era remains on disk: `~/.ship/kernel/events.db` and `~/.ship/actors/{id}/` still get created by init paths even though all events consolidated into `platform.db` (see `crates/core/runtime/src/persistence/snapshot.rs:10`, `docs/ship-kernel/references/docs/kernel.md:67`).
- `crates/core/runtime/src/templates/` ships 10 stub templates (`ADR.md`, `FEATURE.md`, `ISSUE.md`, `MCP.toml`, `NOTE.md`, `PERMISSIONS.toml`, `RELEASE.md`, `RULE.md`, `SPEC.md`, `VISION.md`) with zero references in the codebase — leftover from a removed stub-generator flow.

The result: a home directory that rewards the original customization vectors, disadvantages the new ones, and has no enforcement mechanism preventing further drift.

## Decision

### 1. Two buckets at `~/.ship/`: `library/` and `machine/`

```
~/.ship/
  library/                      ← portable user scope: personal library across projects
    agents/
    skills/
    collections/                ← renamed from schemas/
    views/
    pipelines/
    triggers/
    rules/
    permissions/
    themes/
    mcp/                        ← promoted from mcp.jsonc (single-file mcp/config.jsonc for now)
    settings.jsonc              ← portable user preferences

  machine/                      ← never portable; runtime state tied to this host
    db/
      platform.db (+ -shm + -wal)
      data.db (+ -shm + -wal)
      archive/                  ← platform.db.bak-* snapshots
    cache/
    crashes/
    logs/                       ← shipd.log lives here
    sessions/
    actors/                     ← runtime actor state (not per-actor DBs)
    projects/                   ← workspace links; absolute paths, machine-local
    daemon/                     ← IPC handshake: port, token, pid, socket
    local.jsonc                 ← machine-specific overrides (worktrees.dir, etc.)
```

No third bucket. `config` is not a peer category — portable config lives at `library/settings.jsonc`, machine-local config lives at `machine/local.jsonc`. The bucket split answers *"what moves with what"*, not *"what kind of thing is it"*.

### 2. Canonical customization-vector list, applied uniformly at every authoring scope

```
agents · skills · collections · views · pipelines · triggers · rules · permissions · themes · mcp
```

This list is a Rust enum (`Vector`) in `crates/core/runtime/src/home/vector.rs`. Project `.ship/` mirrors the same top-level vector list — identical shape at project and user scopes makes precedence obvious:

**Precedence (lowest → highest):** built-in defaults → user (`library/`) → project (`.ship/`) → project local overrides (gitignored).

Adding a new customization vector is a one-line enum change, which guarantees uniform treatment across scopes, loaders, lint rules, and docs.

### 3. Daemon IPC handshake is a named concept: `machine/daemon/`

The daemon on start writes all four files atomically:

- `port` — TCP port the daemon bound.
- `token` — authentication token for CLI/MCP/hook callers.
- `pid` — process ID.
- `socket` — optional Unix socket path (if used).

On stop, all four are removed. Stale files signal crash. Directory permission `0700`; `token` file `0600`. One well-known location for every caller to discover the running daemon.

### 4. `ShipHome` is the only way to compute paths under `~/.ship/`

```rust
// crates/core/runtime/src/home/mod.rs
pub struct ShipHome { root: PathBuf }

impl ShipHome {
    pub fn library_dir(&self) -> PathBuf { … }
    pub fn machine_dir(&self) -> PathBuf { … }
    pub fn vector_dir(&self, scope: Scope, v: Vector) -> PathBuf { … }
    pub fn db_path(&self, db: Db) -> PathBuf { … }
    pub fn daemon_file(&self, f: DaemonFile) -> PathBuf { … }
}
```

Enforcement:

- A lint rule `runtime-raw-ship-path` in `crates/core/lint/src/rules/` flags any `.join(".ship")`, `home_dir().join(".ship/…")`, or string-literal `"~/.ship/…"` outside `crates/core/runtime/src/home/`.
- Migration of the 40+ existing call sites is acceptance criteria for the restructure job.

### 5. JSONC is the one config format

Convert `config.toml` → `settings.jsonc` (portable keys) and `local.jsonc` (machine-specific keys). TOML support is removed from the runtime config loader. JSONC everywhere means: comments work uniformly, JSON Schema validates everything, no format-specific parser code paths.

### 6. Typed environment variable surface

- Source of truth: `crates/core/runtime/src/env.rs` — typed constants for every `SHIP_*` variable plus a `read(Var)` accessor.
- Reference page: `apps/docs/src/content/docs/reference/env-vars.md` — published catalog with category, type, default, and stability notes.
- Enforcement: lint rule `runtime-raw-env-var` flags `env::var("SHIP_*")` outside `env.rs`.
- Category taxonomy: configuration (`SHIP_HOME`, `SHIP_API_URL`); runtime identity (`SHIP_AGENT_ID`, `SHIP_WORKSPACE_ID`, `SHIP_SESSION_ID`, `SHIP_MESH_ID`, `SHIP_ACTOR_ID`); dev/debug (`SHIP_EXECUTOR`, `SHIP_TMUX_TESTS`, `SHIP_DEFAULT_TERMINAL`); provider detection (read-only: `CODEX_*`, `GEMINI_*`, `CURSOR_*`).

`SHIP_HOME` is the single root override. No per-bucket `SHIP_LIBRARY_DIR` / `SHIP_MACHINE_DIR` — splitting buckets defeats the shape discipline the layout exists to enforce. Opt-in overrides can land later if sandbox scenarios require them.

### 7. Dead-architecture sweep, same change

The restructure is the natural time to remove confirmed cruft:

- Delete `crates/core/runtime/src/templates/` (10 files, zero references).
- Remove the `kernel/events.db` init path and delete `~/.ship/kernel/` on migration.
- Delete legacy per-actor DB dirs under `~/.ship/actors/{id}/events.db`; the `actors/` concept survives in `machine/actors/` for *current* actor state only.
- Delete stale `.ship/hooks/log-failed-jobs.jsonc` (duplicated in `triggers/`) and any other remnants of the hooks-dir rename.

The `platform.db` user-content table drop (`adr`, `note`, `target`, `capability`, `branch_context`) is a **separate job** with a binary completion bar — see the referenced job spec. Included here only as a cross-reference, not as this ADR's scope.

### 8. Migration strategy

One-shot on `shipd` start:

1. Detect layout version (flag file `machine/layout-version` or absence-of-library-dir).
2. If pre-split layout: stop if another daemon instance is running; otherwise move directories into the new buckets inside a single transaction-equivalent (all-or-rollback via temp-dir staging).
3. Convert `config.toml` → `settings.jsonc` + `local.jsonc` in the same pass.
4. Write `machine/layout-version`.
5. If partial legacy detected after migration completes (user moved something manually): hard error with a clear remediation message. No silent repair.

## Consequences

### Positive

- **New customization vectors are first-class from the moment they're added.** The enum forces uniform scoping; no more "we added it at project scope only and forgot user scope for six months."
- **Portability and sync are cleanly scoped.** `library/` is the set of things that follow a user; `machine/` is the set that doesn't. Syncing the former across machines is trivial.
- **One place to look for everything.** Want daemon handshake state? `machine/daemon/`. Want the user's personal agents? `library/agents/`. Want machine-local DBs? `machine/db/`.
- **Enforced.** Raw-path and raw-env lint rules prevent silent regression. A new customization vector that skips the enum fails review.
- **Documented.** Env-var catalog and scope-convention reference mean agents and contributors stop guessing.

### Negative

- **One-time migration touches every installed `~/.ship/`.** Bug in the migration logic is user-visible. Mitigant: temp-dir staging + rollback on failure; hard-error on partial state.
- **40+ call sites change.** Mechanical, but a single job.
- **Breaking env-var or layout changes after this land become painful.** That's the point of the freeze — launch with the right shape, not the expedient one.
- **Third-party tooling** that reads `~/.ship/platform.db` directly breaks. There is no such tooling today; this is a non-event pre-launch, impossible post-launch.

## Alternatives Considered

### XDG Base Directory Spec as the native layout

Linux-native: `$XDG_CONFIG_HOME/ship/`, `$XDG_DATA_HOME/ship/`, `$XDG_STATE_HOME/ship/`, `$XDG_RUNTIME_DIR/ship/`. Correct semantically — daemon IPC in particular belongs in `$XDG_RUNTIME_DIR` (tmpfs, auto-cleaned on logout). Rejected for v1: the cross-platform story (macOS `~/Library/Application Support/ship/`, Windows `%APPDATA%\ship\`) requires per-platform mapping code, and we still need a single `SHIP_HOME` override for Flatpak/sandbox scenarios. Revisit in v1.x once the `ShipHome` abstraction has stabilized — the split itself already anticipates the XDG boundary.

### Three buckets: `config/ state/ content/`

Initially proposed. Rejected because `config` is not orthogonal to scope — portable config and machine-local config are two different things that belong inside their respective buckets, not in a separate config bucket. Two buckets map cleanly to "moves with user" vs "stays on machine."

### Keep `config.toml` as TOML

Rejected. JSONC is the house style (`ship.jsonc`, `mcp.jsonc`, `permissions.jsonc`, `hooks.jsonc`, collection schemas). TOML is the single outlier. Keeping it means maintaining two config parsers forever to support one legacy file.

### Per-bucket env-var overrides (`SHIP_LIBRARY_DIR`, `SHIP_MACHINE_DIR`)

Rejected for v1. Lets users scatter buckets across the filesystem, which defeats the shape discipline. Single `SHIP_HOME` with fixed subdirs is opinionated and intentional.

### Keep `hook.token` / `network.pid` at root

Rejected. Daemon handshake is a distinct concept that deserves a named location. Scattered files at root treat IPC metadata as peer of user content.

## Migration Plan

Detailed in the restructure job spec. Summary:

1. Land `Vector` enum, `ShipHome`, `Env` modules and their lint rules — all new code, no moves yet.
2. Migrate 40+ call sites from raw paths to `ShipHome`; migrate env reads to `env.rs`. CI passes with new lint rules enabled.
3. Ship the one-shot home migrator; land layout changes on `shipd` start.
4. Delete dead dirs (`templates/`, `kernel/`, stale `actors/` subtrees).
5. Publish env-var catalog and scope-convention reference pages.

## How to Measure Success

- `~/.ship/` contains exactly two top-level dirs: `library/` and `machine/`.
- Every customization vector — old and new — is a sibling directory at both user and project scopes.
- `rg "home_dir\(\).join\(\"\.ship"` returns zero hits outside `crates/core/runtime/src/home/`.
- `rg 'env::var\("SHIP_'` returns zero hits outside `crates/core/runtime/src/env.rs`.
- No `.toml` files under `~/.ship/` or project `.ship/` after migration.
- `ship daemon start` on a pre-split `~/.ship/` migrates cleanly; `ship daemon start` on an already-migrated home is a no-op.
- Env-var reference page enumerates every `SHIP_*` variable the runtime reads.
