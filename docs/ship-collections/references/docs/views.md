---
group: Collections
title: Views
section: guide
order: 4
---

# Building views from collection data

Collections expose their schemas + items through `ship.collections.*`
ops, and SUI views consume those ops the same way they consume any
other live data source — declare a `<data live>` source and let the
runtime resubscribe when the inputs change.

See the [SUI views](/views/) section for the full template + controller
surface, and [data subscriptions](/views/data-subscriptions/) for how
`<data live>` envelopes work.

## Read a collection

```sui
<!-- views/agents-list/page.sui -->
<data name="agents" source="ship.collections.items({ id: 'agents' })" live />

{#while-loading of agents()}
  <p>Loading…</p>
{/while-loading}

{#for item of agents().value.items key={item.id}}
  <article>
    <h3>{item.name}</h3>
    <p class="muted">{item.role}</p>
  </article>
{/for}
```

The daemon serves the items from the collection's schema; the SSE
channel pushes patches whenever an item lands or changes.

## Schema-driven forms

`ship.collections.schema({ id })` returns the field list — pair it
with `<sui-input>` / `<sui-select>` to build a form whose fields
follow the schema:

```sui
<data name="schema" source="ship.collections.schema({ id: 'agents' })" />

<form on:submit|prevent={() => create()}>
  {#for f of schema().value.fields}
    <label>
      {f.label}
      <sui-input bind:value={form[f.name]} type={f.type} />
    </label>
  {/for}
  <sui-button type="submit" tone="primary">Create</sui-button>
</form>
```

The controller wires `form` and `create()` in its `scope:` factory.

## Mutate

Collection mutations go through the standard SDK call surface —
`ship.collections.create / update / delete` — invoked from the
controller (not the template). The `<data live>` subscription
re-emits as soon as the daemon persists the change, so the list
re-renders without an explicit invalidation step.

See [collections op catalog](./querying.md) for the full call list.
