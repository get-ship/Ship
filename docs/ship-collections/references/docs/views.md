---
group: Collections
title: Views
section: guide
order: 4
---

# Building views with collections

Views in `.ship/views/` can consume collection data using the same tools as daemon-internal views: Jinja URL macros, ship-elements, and Datastar.

## The view SDK

There is no separate SDK library. The SDK is:

1. **Jinja URL macros** — typed endpoint builders injected at serve time
2. **Ship-elements** — web components for UI (inputs, selects, buttons, chips)
3. **Datastar** — declarative data binding and SSE

Views are `.html.j2` files processed by the daemon's Jinja engine. They inherit all registered template macros.

## Using URL macros

Import endpoint builders at the top of your view:

```jinja
{% from '_urls.html.j2' import create_collection_item, query_collection %}
```

Use them in JavaScript for data fetching:

```html
<script type="module">
  const QUERY_URL = '{{ query_collection("task") }}';
  const items = await fetch(QUERY_URL, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ collection: 'task', limit: 100 }),
  }).then(r => r.json());
</script>
```

Or in Datastar actions for mutations:

```html
<ship-button
  data-on:click="@post('{{ create_collection_item("task") }}',
    {contentType: 'json', body: JSON.stringify({data: {title: 'New task'}})})">
  Create
</ship-button>
```

## Dynamic URLs

For collection/item IDs known at runtime, use placeholder substitution:

```jinja
const API = {
  create: (col) => '{{ create_collection_item("__COL__") }}'.replace('__COL__', col),
  update: (col, id) => '{{ update_collection_item("__COL__", "__ID__") }}'
    .replace('__COL__', col).replace('__ID__', id),
};
```

## Ship-elements in forms

Use ship-elements for typed inputs that match collection field types:

| Field type | Component | Event |
|-----------|-----------|-------|
| string | `<ship-input>` | `ship-input` |
| markdown | `<ship-textarea>` | `ship-input` |
| number | `<ship-input type="number">` | `ship-input` |
| boolean | `<ship-switch>` | `ship-switch-change` |
| enum | `<ship-select>` + `<ship-option>` | `ship-select-change` |
| date | `<ship-input type="date">` | `ship-input` |
| file | `<ship-input>` | `ship-input` |
| array | `<ship-input>` (JSON string) | `ship-input` |
| object | `<ship-input>` (JSON string) | `ship-input` |

## Live updates via SSE

Subscribe to collection events for real-time updates:

```javascript
const es = new EventSource('/api/runtime/events', { withCredentials: true });
es.onmessage = (e) => {
  const event = JSON.parse(e.data);
  if (event.event_type?.startsWith('collection.')) {
    // Refresh data — the event payload includes collection, item_id, and data
    reload();
  }
};
```

Events emitted on mutations:

| Event type | When |
|-----------|------|
| `collection.item_created` | New item created |
| `collection.item_updated` | Item fields changed |
| `collection.item_deleted` | Item removed |

## Example view

See `.ship/views/data/index.html.j2` for a complete example — a collection browser with table rendering, create/edit forms, and SSE live updates.

## Available URL macros for collections

| Macro | Path |
|-------|------|
| `list_collections()` | `GET /api/collections` |
| `get_collection_schema(collection)` | `GET /api/collections/{collection}/schema` |
| `create_collection_item(collection)` | `POST /api/collections/{collection}/items` |
| `get_collection_item(collection, item_id)` | `GET /api/collections/{collection}/items/{item_id}` |
| `update_collection_item(collection, item_id)` | `PUT /api/collections/{collection}/items/{item_id}` |
| `delete_collection_item(collection, item_id)` | `DELETE /api/collections/{collection}/items/{item_id}` |
| `query_collection(collection)` | `POST /api/collections/{collection}/query` |
