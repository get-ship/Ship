---
group: Architecture
title: Agent Workspaces (CoW + restore) — design
description: Copy-on-write agent workspaces with untracked-inclusive restore, git as an optional module, per-OS via the host adapter.
audience: internal
status: design
---

# Agent workspaces: CoW, restore, and decoupling git

Design doc — not yet built. Decides how a dispatched agent gets its working
directory, how its changes are isolated and **restorable** (including untracked
files), and how that relates to the (unstarted) log-as-truth inversion. Sits on
the `ship-substrate-host` adapter seam established by [agent access](/reference/architecture/agent-access/).

## Today, and what's wrong with it

An agent gets a **git worktree** (`create_worktree` → `git worktree add`). This
assumes **git is load-bearing**: no git repo, no isolated workspace. Problems:

- **Git-coupled.** A project that isn't a git repo (or an app bundle that's pure
  config) can't get an isolated agent workspace.
- **Heavy.** A full worktree is a full checkout — slow to spawn, expensive to
  throw away, N copies for N agents.
- **Can't restore untracked files.** Git only tracks tracked files. An agent that
  writes scratch/build/untracked output leaves it outside git's restore reach —
  the operator can't cleanly roll back "everything this agent touched."

## Principle: git is a module, not the substrate

The workspace substrate is **copy-on-write**, provided by the host adapter. **Git
is an optional complementary module** layered *on top* when a project wants the
branch/PR flow — it is never assumed. The CoW workspace works for a git repo, a
plain directory, or an app bundle identically.

## The primitive

A new `ship-substrate-host` capability, per-OS like `process_container`/`peer_cred`:

```rust
trait HostOs {
    /// Give agent `id` a copy-on-write workspace over `base`. Cheap to create,
    /// cheap to discard; the agent's writes are physically isolated from `base`.
    fn provision_workspace(&self, id: &str, base: &Path) -> io::Result<Workspace>;
}
```

`Workspace` exposes the agent's root path and a handle to its **change set** (the
isolated writes), which is what restore and review operate on.

### Per-OS mechanism (the adapter choice, made per platform)

| OS | CoW mechanism | "Agent's diff" is… | Notes |
|----|---------------|--------------------|-------|
| **Linux** | `fuse-overlayfs` (rootless) or kernel overlay in a userns | the **upper dir** — a free, exact byte-diff | rootless `mount -t overlay` needs CAP_SYS_ADMIN; fuse-overlayfs avoids it. This host has both. |
| **macOS** | APFS `clonefile()` (CoW clone of the tree) | *derived*: clone vs base, or the event index (below) | no overlay/upper-layer; the clone is independent, so the diff isn't free |
| **Windows** | ReFS block-cloning (Dev Drive) / copy fallback | *derived*, as macOS | block-clone where available, plain copy otherwise |

The key asymmetry: **only Linux overlay gives the diff as a free byproduct** (the
upper dir). On clone-based CoW (macOS/Windows) the diff must be *derived*. That is
exactly why restore should not lean solely on the filesystem.

## Restore semantics — the fork

"Restore the changes an agent made" has three possible foundations:

1. **Filesystem layers** — restore = discard/snapshot the overlay upper dir.
   Captures *everything* (tracked, untracked, binary, config) for free on Linux.
   But it's Linux-overlay-specific and coarse (snapshot granularity, not
   per-change), and doesn't exist on clone-based CoW.
2. **Event log (#91 log-as-truth)** — every change is a semantic event; files are
   projections; restore = revert/replay to a point. Rich, queryable, per-change,
   auditable, **cross-platform**. But #91 hasn't happened (config is file-first
   today), and a pure event model doesn't naturally capture arbitrary
   untracked/binary writes unless every write is intercepted.
3. **Hybrid (recommended)** — CoW gives cheap workspaces + cheap **snapshots**;
   an **event-per-change index** records *what changed when* and points at the
   backing snapshot. Restore is then either coarse (revert to snapshot) or
   semantic (revert a specific change). Untracked files are covered because the
   snapshot is physical; the event index makes it queryable and works uniformly
   across OSes where the upper-dir trick doesn't exist.

The hybrid is the cross-platform-honest answer: it doesn't depend on the
Linux-only "upper dir = diff" property, and it gives the **untracked-inclusive
restore** that's explicitly wanted.

## Relationship to log-as-truth (#91)

These are **complementary, staged**, not the same project:

- **#91** inverts *config* (agents/skills/pipelines) so the event log is the
  source and files are projected. It is about Ship's own configuration surfaces.
- **This** is about an *agent's arbitrary file changes* in its workspace —
  including files Ship doesn't model (source, build output, scratch).

The per-change event index here is the same *shape* as #91 (changes as events)
and can share machinery, but it doesn't require #91 to land first. If #91 lands,
config changes flow through the same log; until then, the workspace index covers
the agent's filesystem writes regardless of whether Ship models the file.

## Phasing

- **P0** — `provision_workspace` on `HostOs` + Linux `fuse-overlayfs` backend;
  supervisor spawns agents into a CoW workspace instead of a git worktree.
  Git-optional from day one (base may or may not be a repo).
- **P1** — snapshot + discard (coarse restore); operator can roll back
  "everything this agent touched," untracked included.
- **P2** — per-change event index over the workspace; semantic restore + review.
- **P3** — git as an explicit module: when enabled, runs over the workspace for
  the branch/PR flow; the CoW layer and git coexist (git sees a normal tree).
- **P4** — macOS (`clonefile`) + Windows (ReFS) backends behind the same trait.

## Open decisions

- **P0 mechanism on Linux**: `fuse-overlayfs` (portable, slight FUSE overhead) vs
  userns+kernel-overlay (native, fragile across distros). Leaning fuse-overlayfs.
- **Snapshot store location**: under the daemon dir vs alongside the workspace;
  GC policy for discarded agent workspaces.
- **Event index granularity**: per-write (fine, noisy) vs per-step/flush (coarse,
  aligned with agent turns). Leaning per-step to match the agent lifecycle.
- **Interaction with the per-agent container** (cgroup scope): the workspace mount
  lifecycle should track the agent's `ship-agent-<id>` scope — provision on spawn,
  tear down on scope exit.
