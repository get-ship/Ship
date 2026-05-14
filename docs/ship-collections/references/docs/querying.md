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
POST /api/v1/collections/{collection}/query
```

## Request body

```json
{
  "collection": "task",
  "filter": {
    "status": "todo",
    "priority__gte": 2
  },
  "sort": { "priority": -1 },
  "limit": 50,
  "offset": 0
}
```

## Filter operators

Filters use Django-style suffix keys: the operator is appended to the field
name with a double underscore (`field__op`). A bare field name means equality.

### Equality (implicit)

```json
{ "status": "accepted" }
```

### Comparison

| Operator key | Meaning | Example |
|----------|---------|---------|
| `field__gt` | Greater than | `{"priority__gt": 2}` |
| `field__gte` | Greater or equal | `{"priority__gte": 2}` |
| `field__lt` | Less than | `{"priority__lt": 5}` |
| `field__lte` | Less or equal | `{"priority__lte": 5}` |
| `field__ne` | Not equal | `{"status__ne": "done"}` |

### Set membership

| Operator key | Meaning | Example |
|----------|---------|---------|
| `field__in` | Value in set | `{"status__in": ["todo", "doing"]}` |
| `field__nin` | Value not in set | `{"status__nin": ["done"]}` |

### Existence

```json
{ "assignee__exists": true }
{ "assignee__exists": false }
```

### Text search

```json
{ "title__contains": "test" }
```

Performs a SQL LIKE `%term%` search. Case-insensitive (SQLite `LIKE` is
case-insensitive for ASCII).

### Null

```json
{ "assignee": null }
```

### OR conditions

```json
{
  "_or": [
    { "status": "todo" },
    { "priority__gte": 3 }
  ]
}
```

Use the `_or` key (single underscore) with an array of filter objects. Each
condition in the array supports the full `field__op` operator set — multiple
keys within one condition object are AND-ed together.

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
query_docs(collection: "task", filter: {"status": "todo", "priority__gte": 2}, sort: {"priority": -1}, limit: 20)
```
