---
group: Collections
title: Querying
section: reference
order: 3
audience: internal
---

# Querying

Query items with structured filters, sorting, and pagination.

## Endpoint

```
POST /api/collections/{collection}/query
```

## Request body

```json
{
  "collection": "task",
  "filter": {
    "status": "todo",
    "priority": { "$gte": 2 }
  },
  "sort": { "priority": -1 },
  "limit": 50,
  "offset": 0
}
```

## Filter operators

### Equality (implicit)

```json
{ "status": "accepted" }
```

### Comparison

| Operator | Meaning | Example |
|----------|---------|---------|
| `$gt` | Greater than | `{"priority": {"$gt": 2}}` |
| `$gte` | Greater or equal | `{"priority": {"$gte": 2}}` |
| `$lt` | Less than | `{"priority": {"$lt": 5}}` |
| `$lte` | Less or equal | `{"priority": {"$lte": 5}}` |
| `$ne` | Not equal | `{"status": {"$ne": "done"}}` |

### Set membership

| Operator | Meaning | Example |
|----------|---------|---------|
| `$in` | Value in set | `{"status": {"$in": ["todo", "doing"]}}` |
| `$nin` | Value not in set | `{"status": {"$nin": ["done"]}}` |

### Existence

```json
{ "assignee": { "$exists": true } }
{ "assignee": { "$exists": false } }
```

### Text search

```json
{ "title": { "$contains": "test" } }
```

Performs a SQL LIKE `%term%` search. Case-sensitive.

### Null

```json
{ "assignee": null }
```

### OR conditions

```json
{
  "$or": [
    { "status": "todo" },
    { "status": "doing" }
  ]
}
```

Each condition in the `$or` array supports equality filters. Operator objects inside `$or` are not yet supported.

## Sorting

```json
{ "sort": { "created_at": -1 } }
```

`1` for ascending, `-1` for descending. Multiple sort fields are applied in order. Default: `created_at DESC`.

## Pagination

| Field | Default | Max | Description |
|-------|---------|-----|-------------|
| `limit` | 50 | 500 | Maximum items to return |
| `offset` | 0 | — | Skip this many items |

## Response

```json
{
  "ok": true,
  "data": [
    {
      "id": "t1",
      "collection": "task",
      "data": { "title": "Write tests", "status": "todo", "priority": 1 },
      "created_by": "alice",
      "created_at": "2026-04-14T10:00:00Z",
      "updated_at": "2026-04-14T10:00:00Z"
    }
  ],
  "count": 1
}
```

## MCP tool

Agents use the `query_docs` tool with the same filter/sort/limit parameters:

```
query_docs(collection: "task", filter: {"status": "todo"}, sort: {"priority": -1}, limit: 20)
```
