---
group: Concepts
title: Why a runtime
description: Ship is a runtime, not a wrapper. Event-sourced, supervises agents, gives you observability for free.
order: 4
---

Ship is a runtime, not a CLI wrapper. That choice has consequences for what you can build on top.

## Event-sourced

Every state change in Ship is an event on a local log — every artifact written, every skill invocation, every approval gate, every step transition. Two things fall out of this:

- **Pipelines are resumable.** A run that crashed at 2am can pick up where it left off, because its state can be replayed from the log.
- **The audit trail comes for free.** You don't add tracing — every record is already there.

## Supervises agents

When you kick off a pipeline, Ship dispatches each step, watches its lifecycle, holds gates open for human approval, and routes completion signals downstream. Agents run as supervised actors, not detached subprocesses — the system knows what came before and what should come next.

## Observability without instrumentation

The dashboard shows what every agent is doing, live. Which run produced which artifact, which step is waiting on a human, which agent owns which workspace — all of it is a query against the event log. You don't instrument anything; it's a property of the architecture.

This is the difference between Ship and a script that calls a CLI agent. The runtime gives you a system you can build a real app on top of.
