---
title: Why a runtime
description: Ship is a runtime, not a CLI wrapper — event-sourced, supervises agents, and gives you observability for free.
section: Concepts
order: 4
kicker: The mental model
tags: [concepts, runtime, observability]
related: [what-is-ship]
---

Ship is a runtime, not a CLI wrapper — and that choice decides what you can build on top.

## Event-sourced

Every state change is an event on a local log: every artifact written, every skill invocation, every approval gate, every step transition. Two things fall out of that:

- **Pipelines are resumable.** A run that crashed at 2am can pick up where it left off, because its state replays from the log.
- **The audit trail comes for free.** You don't add tracing — every record is already there.

## It supervises agents

Kick off a pipeline and Ship dispatches each step, watches its lifecycle, holds gates open for human approval, and routes completion signals downstream. Agents don't run in isolation; they run inside a system that knows what came before and what comes next.

## Observability without instrumentation

What each agent is doing, which run produced which artifact, which step is waiting on a human — all of it is a query against the event log. You don't instrument anything; it's a property of the architecture.

That's the difference between Ship and a script that shells out to a CLI agent: the runtime gives you a system you can build a real app on.
