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
    mcp.jsonc                   ← user-scope MCP config (file, mirrors project shape)
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
agents · skills · collections · views · pipelines · triggers · rules · permissions · themes
```

This list is a Rust enum (`Vector`) in `crates/core/runtime/src/home/vector.rs` — nine **directory** vectors. Project `.ship/` mirrors the same top-level shape — identical at project and user scopes makes precedence obvious:

**Precedence (lowest → highest):** built-in defaults → user (`library/`) → project (`.ship/`) → project local overrides (gitignored).

Adding a new directory-vector is a one-line enum change, which guarantees uniform treatment across scopes, loaders, lint rules, and docs.

**`mcp` is intentionally not a directory vector.** At every scope MCP config is a single JSONC file (`<scope>/mcp.jsonc`). The original draft promoted it to a `mcp/` directory for symmetry; that was overengineering — a single file works at both scopes and matches existing project shape.

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

- Grep guard `scripts/lint/check-raw-ship-paths.sh` (run via `just check-raw`) flags any raw `.join(".ship")` or `home_dir().join(".ship/…")` outside `crates/core/runtime/src/home/`. Hard-fails CI on regression.

### 5. JSONC is the one config format

Two config files: `library/settings.jsonc` (portable keys, e.g. `[identity]`) and `machine/local.jsonc` (machine-specific keys, e.g. `worktrees.dir`). Project-scope manifest is `.ship/ship.jsonc`. The runtime does not parse TOML for any Ship config (legacy `config.toml`/`ship.toml` support is gone — pre-launch product, no migration). Provider-specific TOML (e.g. `.codex/config.toml`, `.gemini/policies/ship.toml`) is unaffected; that's their format, not Ship's.

### 6. Typed environment variable surface

- Source of truth: `crates/core/runtime/src/env.rs` — typed `Var` enum with one variant per env var the runtime reads, plus an `env::read(Var)` accessor.
- Enforcement: grep guard `scripts/lint/check-raw-env-reads.sh` (run via `just check-raw`) flags `env::var("SHIP_*")` outside `env.rs`. Hard-fails CI on regression.
- Category taxonomy: `Configuration` (`SHIP_HOME`, `SHIP_API_URL`, `SHIP_REGISTRY_URL`); `RuntimeIdentity` (`SHIP_AGENT_ID`, `SHIP_WORKSPACE_ID`, `SHIP_SESSION_ID`, `SHIP_MESH_ID`, `SHIP_ACTOR_ID`, `SHIP_WORKTREE_DIR`, `SHIP_REPO_DIR`); `DevDebug` (`SHIP_DEV`, `SHIP_EXECUTOR`, `SHIP_TMUX_TESTS`, `SHIP_DEFAULT_TERMINAL`, `SHIP_RESPECT_WORKTREE`, `SHIP_DEV_PROJECT_DIR`, telemetry/hook tokens).

`SHIP_HOME` is the single ship-home env override. No per-bucket `SHIP_LIBRARY_DIR` / `SHIP_MACHINE_DIR` — splitting buckets defeats the shape discipline the layout exists to enforce.

`SHIP_DEV_PROJECT_DIR` is the project-root override (replaces the earlier `SHIP_DIR`); used by tests and dev tooling to redirect `runtime::project::get_project_dir(None)` to a tempdir. Production callers never set it.

### 7. Dead-architecture sweep

Stale code removed alongside this layout:

- `crates/core/runtime/src/templates/` (zero references).
- `kernel/events.db` init path; `~/.ship/kernel/` no longer created.
- Legacy per-actor DB dirs under `~/.ship/actors/{id}/events.db`; the `actors/` concept survives in `machine/actors/` for *current* actor state only.

The `platform.db` user-content table drop (`adr`, `note`, `target`, `capability`, `branch_context`) is a **separate job**.

### 8. Initialization, not migration

Pre-launch product, no installed user base. The runtime does **not** carry a one-shot home-layout migrator — that was 674 lines of code solving a problem we don't have. On daemon start, `runtime::home::ensure_layout(&ShipHome)` is called from `apps/shipd/src/ship_runtime.rs` and runs `mkdir -p` for the canonical bucket tree (`library/`, `machine/`, `machine/{db,daemon,logs,cache,sessions,actors,projects,crashes}/`). Idempotent; no detection logic, no rollback, no version marker. Fresh installs and existing per-developer installs converge on the same shape.

If we ship to real users with state we need to preserve later, a successor ADR captures that migration story. Until then: no migrator code in the runtime.

The runtime "hooks" module (`crates/core/runtime/src/hooks/`) is **trigger infrastructure** with a stale name — it loads file/cron/webhook trigger definitions from `<scope>/triggers/` (matching the Vector enum). The legacy `<scope>/hooks/` directory is no longer read.

## Consequences

### Positive

- **New customization vectors are first-class from the moment they're added.** The enum forces uniform scoping; no more "we added it at project scope only and forgot user scope for six months."
- **Portability and sync are cleanly scoped.** `library/` is the set of things that follow a user; `machine/` is the set that doesn't. Syncing the former across machines is trivial.
- **One place to look for everything.** Want daemon handshake state? `machine/daemon/`. Want the user's personal agents? `library/agents/`. Want machine-local DBs? `machine/db/`.
- **Enforced.** Raw-path and raw-env lint rules prevent silent regression. A new customization vector that skips the enum fails review.
- **Documented.** Env-var catalog and scope-convention reference mean agents and contributors stop guessing.

### Negative

- **40+ call sites changed** to route through typed accessors. Mechanical, done in batches.
- **Breaking env-var or layout changes after launch become painful.** That's the point of locking in early — launch with the right shape, not the expedient one.
- **No backward compatibility for any pre-ADR per-developer state.** Pre-launch we accept this; we wipe + re-init rather than carry migrator code forever. Post-launch a different ADR will own real-user migrations.

## Alternatives Considered

### XDG Base Directory Spec as the native layout

Linux-native: `$XDG_CONFIG_HOME/ship/`, `$XDG_DATA_HOME/ship/`, `$XDG_STATE_HOME/ship/`, `$XDG_RUNTIME_DIR/ship/`. Correct semantically — daemon IPC in particular belongs in `$XDG_RUNTIME_DIR` (tmpfs, auto-cleaned on logout). Rejected for v1: the cross-platform story (macOS `~/Library/Application Support/ship/`, Windows `%APPDATA%\ship\`) requires per-platform mapping code, and we still need a single `SHIP_HOME` override for Flatpak/sandbox scenarios. Revisit in v1.x once the `ShipHome` abstraction has stabilized — the split itself already anticipates the XDG boundary.

### Three buckets: `config/ state/ content/`

Initially proposed. Rejected because `config` is not orthogonal to scope — portable config and machine-local config are two different things that belong inside their respective buckets, not in a separate config bucket. Two buckets map cleanly to "moves with user" vs "stays on machine."

### Keep `config.toml` as TOML

Rejected. JSONC is the house style (`ship.jsonc`, `mcp.jsonc`, `permissions.jsonc`, collection schemas). TOML is the single outlier. Both `~/.ship/config.toml` and project-scope `ship.toml` reads have been removed from the runtime — JSONC manifests only.

### Per-bucket env-var overrides (`SHIP_LIBRARY_DIR`, `SHIP_MACHINE_DIR`)

Rejected for v1. Lets users scatter buckets across the filesystem, which defeats the shape discipline. Single `SHIP_HOME` with fixed subdirs is opinionated and intentional.

### Keep `hook.token` / `network.pid` at root

Rejected. Daemon handshake is a distinct concept that deserves a named location. Scattered files at root treat IPC metadata as peer of user content.

## How to Measure Success

- `~/.ship/` contains exactly two top-level dirs: `library/` and `machine/`.
- Every directory-vector is a sibling directory at both user and project scopes.
- `rg "home_dir\(\).join\(\"\.ship"` returns zero hits outside `crates/core/runtime/src/home/`.
- `rg 'env::var\("SHIP_'` returns zero hits outside `crates/core/runtime/src/env.rs`.
- No `.toml` files under `~/.ship/` or project `.ship/` (other than provider-specific TOML like `.codex/config.toml`).
- A fresh `shipd` start (no prior `~/.ship/`) populates the canonical layout and writes daemon IPC files at `machine/daemon/{port,token,pid}`.
- The `apps/shipd/tests/smoke.rs::fresh_daemon_writes_canonical_home_layout` regression test passes — it asserts the layout shape and zero legacy paths after a real daemon start.
- Operational reference: `docs/ship-runtime/references/docs/home-layout.md`.
