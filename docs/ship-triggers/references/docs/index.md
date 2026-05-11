---
group: Triggers
order: 1
title: Triggers
description: Reactive rules. Watch for an event, run an action.
---

A **trigger** is a small declarative rule: "when X happens, do Y." You drop a JSONC file into `.ship/` and the runtime starts listening. No code, no glue — just a description of cause and effect.

## What you can listen to

<ship-card padding="md">
<strong slot="header">Runtime events</strong>
Anything happening in your Ship project — a job finished, a collection item was created, an artifact was uploaded.
</ship-card>

<ship-card padding="md">
<strong slot="header">Webhooks</strong>
An external service POSTing to a path your trigger registers — GitHub, Stripe, your own services.
</ship-card>

<ship-card padding="md">
<strong slot="header">Files changing</strong>
A path or glob you watch — the trigger fires on create, modify, or delete.
</ship-card>

<ship-card padding="md">
<strong slot="header">Cron schedules</strong>
A standard cron expression. Runs at the specified time, no daemon nag needed.
</ship-card>

## What a trigger can do

Four built-in actions:

- **Insert into a collection** — capture a record when something happens.
- **Update collection items** — change matching rows in response to an event.
- **Emit a derived event** — re-emit a transformed version for other triggers to listen to.
- **Run a pipeline** — kick off multi-step work in response to anything.

Actions run in order. Each one has access to the triggering event's data via templates so the trigger can reshape inputs into the action's expected form.

## A small example

A trigger that runs the nightly digest pipeline at 2 a.m.:

```jsonc
{
  "name": "nightly-digest",
  "trigger": {
    "source": {
      "type": "cron",
      "expression": "0 2 * * *",
      "timezone": "America/New_York"
    }
  },
  "actions": [
    { "type": "pipeline.trigger", "pipeline": "nightly-digest" }
  ]
}
```

A trigger that captures GitHub pushes into a collection:

```jsonc
{
  "name": "on-github-push",
  "trigger": {
    "source": {
      "type": "webhook",
      "path": "/webhooks/github",
      "secret": "env:GH_WEBHOOK_SECRET"
    }
  },
  "actions": [
    {
      "type": "collection.insert",
      "collection": "pushes",
      "data": { "ref": "{{ event.payload.body.ref }}" }
    }
  ]
}
```

Triggers hot-reload — edit the file, save, the runtime picks it up. No restart.
