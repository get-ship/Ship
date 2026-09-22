---
title: Security & isolation
description: Where your data lives, what agents are allowed to touch, and how work is isolated — the trust model in one page.
section: Security
order: 1
kicker: Security
tags: [security, permissions, isolation]
related: [permissions, local-first]
status: ready
---

Ship's trust model rests on three things: your data stays local, agents act under explicit permissions, and work runs in isolation.

## Your data is local

The runtime, your databases, and your data all live on your machine — there's no Ship server in the path. Nothing leaves your laptop unless you publish or opt into cloud sync. See [Local-first](/docs/local-first/).

## Agents act under permissions

Every agent runs under a permission preset that decides what it can do without asking — `ship-readonly`, `ship-standard`, `ship-autonomous`, or `ship-elevated` — and you can layer allow/deny/ask rules on top. The elevated preset is the only one that unlocks `git push` and publish; force-push and hard resets stay blocked even there. Full detail in [Permissions](/docs/permissions/).

## Work runs in isolation

- **Spaces are process-isolated**, and an agent's context is scoped to its space — agents don't reach across spaces. See [Spaces](/docs/spaces/).
- **Pipeline runs get an isolated workspace** (a separate worktree); changes merge back to your parent branch only when a gate passes.

Together: local data you own, agents bounded by presets you choose, and dispatched work fenced off until you approve the merge.
