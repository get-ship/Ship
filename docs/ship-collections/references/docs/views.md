---
group: Collections
title: Views
section: guide
order: 4
---

# Building views with collections

Views in `.ship/views/` render collection data server-side. The daemon passes
schemas and items as Jinja context — no JavaScript data fetching needed.

## How it works

1. Create `.ship/views/my-view/index.html.j2`
2. The daemon renders it through Jinja, injecting collection data as context
3. Ship-elements render as custom elements in the browser
4. Datastar handles mutations (`@post`, `@delete`)
5. SSE subscription reloads on changes (4 lines of JS)

## Template context

Every `.ship/views/` Jinja template receives:

| Variable | Type | Description |
|----------|------|-------------|
| `view_name` | string | The view's directory name |
| `collections` | array | All registered schemas with id, name, field_count, fields |
| `active_collection` | string | ID of the first collection |
| `schema` | object | Full schema for the active collection |
| `items` | array | Items in the active collection (up to 200) |
| `total_items` | number | Item count |

## URL macros

Import typed endpoint builders:

```jinja
{% raw %}{% from '_urls.html.j2' import create_collection_item, delete_collection_item %}{% endraw %}
```

Use in Datastar actions:

```html
{% raw %}<ship-button data-on:click="@delete('{{ delete_collection_item(active_collection, item.id) }}')">
  Delete
</ship-button>{% endraw %}
```

Available macros:

| Macro | Method | Path |
|-------|--------|------|
| `list_collections()` | GET | `/api/collections` |
| `get_collection_schema(collection)` | GET | `/api/collections/{collection}/schema` |
| `create_collection_item(collection)` | POST | `/api/collections/{collection}/items` |
| `update_collection_item(collection, item_id)` | PUT | `/api/collections/{collection}/items/{item_id}` |
| `delete_collection_item(collection, item_id)` | DELETE | `/api/collections/{collection}/items/{item_id}` |
| `query_collection(collection)` | POST | `/api/collections/{collection}/query` |

## Rendering fields

```html
{% raw %}{% if field_def.field_type == "enum" %}
  <ship-chip>{{ val }}</ship-chip>
{% elif field_def.field_type == "boolean" %}
  <ship-badge variant="{{ 'success' if val else 'secondary' }}">{{ "Yes" if val else "No" }}</ship-badge>
{% elif field_def.field_type == "number" %}
  <span class="mono">{{ val }}</span>
{% elif field_def.field_type == "array" and val is iterable and val is not string %}
  {% for v in val[:4] %}<ship-chip>{{ v }}</ship-chip> {% endfor %}
{% else %}
  {{ val }}
{% endif %}{% endraw %}
```

## Create forms

Ship-elements for typed inputs, Datastar `@post` for submission:

```html
{% raw %}<form data-on:submit__prevent="@post('{{ create_collection_item(active_collection) }}',
  {contentType:'json', body: JSON.stringify({data: Object.fromEntries(new FormData(this))})})">
  <ship-input name="title" placeholder="Title"></ship-input>
  <ship-select name="status">
    {% for v in schema.fields.status.values %}
    <ship-option value="{{ v }}">{{ v }}</ship-option>
    {% endfor %}
  </ship-select>
  <ship-button type="submit">Create</ship-button>
</form>{% endraw %}
```

## SSE live updates

```html
<script>
const es = new EventSource('/api/runtime/events', { withCredentials: true });
es.onmessage = e => {
  try { if (JSON.parse(e.data).event_type?.startsWith('collection.')) location.reload(); }
  catch {}
};
</script>
```

## Design tokens

Use tokens from `view-tokens.css` — never hardcode colors or spacing:

```css
color: var(--foreground);
border: 1px solid var(--border);
font-family: var(--font-mono);
padding: var(--space-3);
```

## Example

See `.ship/views/data/index.html.j2` for a complete collection browser.
