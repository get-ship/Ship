---
title: "Schemas"
sidebar:
  label: "Schemas"
  order: 2
---
A schema defines the shape of items in a collection. Schemas are JSONC files in `.ship/schemas/`.

## Format

```jsonc
// .ship/schemas/adr.jsonc
{
  "id": "adr",
  "name": "Architecture Decision Record",
  "fields": {
    "title": { "type": "string", "required": true },
    "status": { "type": "enum", "values": ["proposed", "accepted", "deprecated"] },
    "context": { "type": "markdown" },
    "decision": { "type": "markdown" },
    "priority": { "type": "number" },
    "approved": { "type": "boolean" },
    "decided_at": { "type": "date" },
    "attachment": { "type": "file" },
    "tags": { "type": "array" },
    "metadata": { "type": "object" }
  }
}
```

## Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Collection identifier. Must be non-empty. |
| `name` | string | no | Human-readable display name. |
| `fields` | object | yes | Field definitions. At least one required. |

## Field types

| Type | Storage | Description |
|------|---------|-------------|
| `string` | TEXT | Short text |
| `markdown` | TEXT | Long-form content |
| `number` | REAL | Integer or float (stored as double) |
| `boolean` | BOOLEAN | true/false (stored as 0/1, coerced on read) |
| `enum` | TEXT | Constrained to `values` list |
| `date` | TEXT | ISO 8601 date string |
| `file` | TEXT | Artifact ID (ULID). Resolves via `GET /api/artifacts/:id`. See [Artifacts](/reference/artifacts/). |
| `array` | TEXT | JSON array (stored as JSON string) |
| `object` | TEXT | JSON object (stored as JSON string) |

## Field options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `required` | boolean | false | Validation rejects items missing this field |
| `default` | any | — | Applied on read when field is absent or null |
| `filterable` | boolean | false | Creates a SQLite index for faster queries |
| `values` | string[] | — | Valid values for enum fields (required for enum type) |

## Schema changes

Adding a field to a schema is safe — Ship adds a column to the table automatically. Existing items get `null` for the new field (or the default value if defined).

Removing a field from the schema hides it from the API but doesn't drop the column. No data is lost.

No migration files. No migration commands. The daemon reconciles on next request.

## Validation

Items are validated against the schema on every write (create and update):

- Required fields must be present and non-null
- Enum fields must match one of the declared values
- Type checking: strings must be strings, numbers must be numbers, etc.
- Unknown fields (not in schema) are silently accepted and stored

Validation errors return 400 with a message listing all violations.
