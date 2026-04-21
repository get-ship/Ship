---
group: Collections
title: Overview
section: guide
order: 1
---

# Collections

A collection is a typed dataset. You define a schema in `.ship/collections/`, and Ship gives you storage, a query API, event emission, and agent access.

## When to use collections

- Structured data that agents and humans create and query (ADRs, tasks, content)
- Any data a view needs to display, filter, and edit
- Anything you'd put in a Notion database or Airtable base

## How it works

1. Define a schema in `.ship/collections/<id>.jsonc`
2. Ship creates a table in `.ship/data.db` with typed columns
3. Use the REST API or MCP tools to create, query, update, delete items
4. Views fetch collection data and render it however they choose
5. Events fire on every mutation — views and agents react in real time

## Quick start

Create a schema:

```jsonc
// .ship/collections/task.jsonc
{
  "id": "task",
  "name": "Task",
  "fields": {
    "title": { "type": "string", "required": true },
    "status": { "type": "enum", "values": ["todo", "doing", "done"] },
    "priority": { "type": "number" }
  }
}
```

Create an item via the REST API:

```bash
curl -X POST /api/collections/task/items \
  -H 'Content-Type: application/json' \
  -d '{"data": {"title": "Write docs", "status": "todo", "priority": 1}}'
```

Query items:

```bash
curl -X POST /api/collections/task/query \
  -H 'Content-Type: application/json' \
  -d '{"collection": "task", "filter": {"status": "todo"}, "sort": {"priority": -1}}'
```

## Architecture

Collections sit on top of the data service — a managed SQLite database at `.ship/data.db`. Each collection gets its own table with real typed columns. The data service is available to any Ship extension, not just collections.

```
.ship/collections/<id>.jsonc    → schema definition
.ship/data.db               → SQLite storage (managed by daemon)
/api/collections/...        → REST API
MCP tools                   → agent access (create_doc, query_docs, etc.)
collection.item_* events    → real-time reactivity
```
