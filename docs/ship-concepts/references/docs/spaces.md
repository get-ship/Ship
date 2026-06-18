---
group: Concepts
title: Spaces
description: A space is a context with its own identity, agents, and data. Switch spaces to switch what your agent sees.
order: 6
---

A **space** is a context with its own identity, agents, and data. It
owns its own view of your project, its own active agents, and its
own event log. Two spaces in the same Ship project never see each
other's tools, files, or decision history unless they explicitly
share a connection.

Spaces are the unit of *context*, not the unit of *isolation*. The
unit of isolation is the actor — every agent, every connector, and
every running view is a supervised actor with its own `cwd`, its own
`space_id`, and its own MCP tool gate. A space is what gives an
actor's scope a name and a window.

## What a space is

- **A name and a path.** Each space is rooted at a directory in your
  project; the path-hash is the space's identity. Move the directory
  and the space follows.
- **A window.** The daemon hosts each space as a webview. Switching
  spaces is a window switch; the state of one space is invisible
  to the others.
- **A default pointer.** Each space remembers which view you last
  had open and resumes there.
- **A scope root.** Every actor the space spawns inherits the
  space's `space_id` on its event log entries. `space_id` is what
  every SDK query keys off of.

## What a space is not

- **Not a security boundary.** A space doesn't keep an agent from
  reading your disk; the actor's `cwd` and tool gate do. Switching
  spaces doesn't grant or revoke permissions.
- **Not a worktree.** A space might map to a git worktree in some
  workflows, but the runtime doesn't track worktrees. Spaces are
  first-class; worktrees are git plumbing.
- **Not a workspace.** The legacy "workspace" concept conflated
  session-identity, dev-worktree, tmux-attachment, and agent-sandbox
  into one primitive. Spaces don't do that. Session is an actor
  dimension; worktree is git; tmux is a transport; the sandbox is
  the actor's tool gate. The space is the context those dimensions
  share.

## Switching spaces

The space switcher in the chrome (or `ship use <space-path>` from the
CLI) flips the active space. The chrome re-resolves against the new
space's identity; the agent's tool gate is re-evaluated against the
new actor scope; the event log re-keys to the new `space_id`. There
is no global "home" space — the daemon boots space-less until it
discovers a project root, and the active space is a per-connection
concern (`ship_substrate::project::resolve_active_space`).

Multiple spaces can be open in parallel (one window per space).
They're independent actors; the daemon doesn't serialise activity
between them.

## Sharing across spaces

Spaces can subscribe to each other's event partitions through
**connectors** — the same mechanism an agent uses to reach an
external service. A connector declared in `.ship/connectors/` is
visible to every actor in every space that has the connector
gated-in; the per-space event partition is still respected, so
the audit trail is intact.

Spaces that need to be physically co-located on the same machine
(say, a dev space and a test space) live in the same `~/.ship/`
under the same daemon. Spaces that need to be on different machines
become part of a team setup, where the team is itself a managed
entity on the cloud side.

## When to add a space

A second space is worth the bookkeeping when:

- You need a clean event partition (audit-by-space, scrub back
  without seeing other spaces' decisions).
- You want different agent permissions in different contexts
  (the home space is permissive; the customer-data space is
  read-only).
- You're working on two unrelated parts of the same project and
  want a clean `cwd` for each.
- You want to share the project with a collaborator and give them
  their own view without seeing your drafts.

A second space is **not** the right tool when:

- You want to keep work-in-progress out of the way. Use a branch.
- You want to test something quickly. Use a worktree.
- You want to hide secrets. Use the secret store, not isolation.

## The space registry

`ship space list` enumerates every space the daemon knows about.
The registry is a projection of `space.added` / `space.removed`
events — the same shape as actors, sessions, and jobs. Adding a
space is an event, not a config edit. Removing a space is an
event, not a delete. The event log is the source of truth.

## What's next

- [Architecture: Runtime Model](../../architecture/runtime-model/) —
  the storage backends, the four primitives, and the decision
  rules for where space state lives.
- [Agents: Permissions](../../agents/permissions/) — what an
  actor's tool gate does, and the four permission presets.
