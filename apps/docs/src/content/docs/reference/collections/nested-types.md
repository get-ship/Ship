---
title: "Nested types"
sidebar:
  label: "Nested types"
  order: 3
---
`array` and `object` fields can carry sub-schemas so the runtime
validates their contents on every write. Items inserted or updated
against a collection must satisfy the nested shape — failures cite the
offending JSON path so callers can pinpoint bad data inside deeply
nested structures.

## Array of T — `items`

Add `items` to an `array` field to declare the schema each element must
satisfy. `items` is itself a full field definition, so elements can be
any supported type (including another `array` or `object`).

```jsonc
// .ship/collections/post.jsonc
{
  "id": "post",
  "fields": {
    "title": { "type": "string", "required": true },
    "tags": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

Inserting `{"title": "x", "tags": ["a", 42, "c"]}` is rejected with an
error naming the bad index — for example,
`field 'tags[1]' must be a string`.

## Object — `properties` and `required_keys`

Add `properties` to an `object` field to declare the schema for each
named property. Add `required_keys` to demand specific keys be present
and non-null. Unknown properties (not listed in `properties`) are
accepted and stored, matching the top-level policy.

```jsonc
{
  "id": "user",
  "fields": {
    "profile": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "age":  { "type": "number" },
        "role": { "type": "enum", "values": ["admin", "user"] }
      },
      "required_keys": ["name"]
    }
  }
}
```

A missing required key surfaces as
`field 'profile.name' is required`. A wrong-typed property surfaces as
`field 'profile.age' must be a number`.

## Nesting

`items` and `properties` recurse, so any combination is valid:

```jsonc
{
  "id": "team",
  "fields": {
    "members": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name":  { "type": "string" },
          "roles": { "type": "array", "items": { "type": "string" } }
        },
        "required_keys": ["name"]
      }
    }
  }
}
```

For an item whose `members[1].roles[0]` is the wrong type, the error
reads `field 'members[1].roles[0]' must be a string`. Indexes use
`[N]`; properties use `.key`.

## Backward compatibility

`items`, `properties`, and `required_keys` are all optional. Existing
schemas that omit them continue to parse and behave permissively — an
`array` with no `items` accepts any JSON array, and an `object` with no
`properties` accepts any JSON object. Add nested schemas only when you
want the runtime to check the shape.

## Validation rules

- Required-key checks treat both missing keys and explicit `null` as
  failures (same policy as top-level `required: true`).
- Validation collects every error per item, then reports them
  together — one bad row produces one error message per offending
  value.
- Errors are prefixed with the JSON path to the value. Top-level field
  paths are bare (`title`); nested paths use `.key` for object members
  and `[N]` for array indices.

## Schema-time validation

The schema itself is checked when `.ship/collections/<id>.jsonc` is
parsed:

- `items` on a non-`array` field is rejected at schema-load time.
- `properties` or `required_keys` on a non-`object` field is rejected
  at schema-load time.
- Every entry in `required_keys` must appear in `properties`.

These guard against typos in the schema before any item is ever
inserted.
