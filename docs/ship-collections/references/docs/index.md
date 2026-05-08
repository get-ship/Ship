---
group: Collections
title: Collections
section: guide
order: 1
---

A **collection** is a typed dataset. You declare its shape in a small JSONC file; Ship gives you storage, queries, real-time events, and agent access — all without writing a database layer.

## When to reach for a collection

<ship-card padding="md">
<strong slot="header">Structured records</strong>
Tasks, ADRs, customer interviews, content drafts — anything you'd otherwise put in a Notion database or an Airtable base.
</ship-card>

<ship-card padding="md">
<strong slot="header">Data your views display</strong>
A view's template can read collection items directly. List, filter, edit — no API plumbing required.
</ship-card>

<ship-card padding="md">
<strong slot="header">A target for triggers</strong>
A trigger can insert into or update a collection in response to an event, turning automation into structured records.
</ship-card>

## A small example

`.ship/collections/task.jsonc`:

```jsonc
{
  "id": "task",
  "name": "Task",
  "fields": {
    "title":    { "type": "string", "required": true },
    "status":   { "type": "enum", "values": ["todo", "doing", "done"] },
    "priority": { "type": "number" }
  }
}
```

That's enough to get a typed table, a CRUD API, an MCP surface for agents, and live events on every change. No code, no migrations.

## What you get

- **Typed storage.** Ship creates a real table for each collection in your project's local database.
- **Query, sort, filter.** A simple structured query language for finding what you want.
- **Live events.** `collection.item_*` events fire on every mutation; views and triggers can listen.
- **Agent access.** Agents can read and write collections via MCP without you wiring up custom tools.

## Next

- [Views over collections](./views/) — how to render collection data in a Ship view.
