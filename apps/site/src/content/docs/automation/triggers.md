---
title: Triggers
description: Reactive rules — when an event happens, run an action. JSONC, no glue code. Pairs with pipelines.
section: Automation
order: 2
kicker: Automation
tags: [automation, triggers]
related: [pipelines]
status: ready
---

A **trigger** is a small declarative rule: "when X happens, do Y." Drop a JSONC file into `.ship/` and the runtime starts listening — no code, no glue, just cause and effect.

## What you can listen to

- **Runtime events** — anything happening in your space: a job finished, a collection item was created, an artifact was uploaded.
- **Webhooks** — an external service POSTing to a path the trigger registers (GitHub, Stripe, your own).
- **File changes** — a path or glob; fires on create, modify, or delete.
- **Cron schedules** — a standard cron expression.

## What a trigger can do

Four built-in actions, run in order, each with access to the triggering event's data:

- **Insert into a collection** — capture a record when something happens.
- **Update collection items** — change matching rows in response to an event.
- **Emit a derived event** — re-emit a transformed version for other triggers.
- **Run a pipeline** — kick off multi-step work in response to anything.

That last one is the dovetail: a trigger watches for the event, a [pipeline](/docs/pipelines/) does the work. A cron trigger that runs a nightly digest pipeline, a webhook that kicks off a review — same shape.
