<scope>
  props: {
    collections = '[]'
    actors = '[]'
    disabled = false
  }
</scope>
<controller>
// <ship-artifact-filters> controller — parses the JSON props into the
// derived row arrays the template binds, and owns the
// `ship-artifact-filter { kind, value }` event surface. No SDK, no
// live data.

import type { Scope } from './ship-artifact-filters.sail'

interface CollectionRow { id: string; name?: string | null }

type FilterKind = 'search' | 'mime' | 'collection' | 'actor' | 'age'

function parseJsonArray<T>(raw: string): T[] {
  try {
    const v = JSON.parse(raw) as unknown
    return Array.isArray(v) ? (v as T[]) : []
  } catch {
    return []
  }
}

export default controller((ctx: Scope, host?: Element) => {
  const collectionRows = derived<CollectionRow[]>(() => parseJsonArray<CollectionRow>(ctx.collections()))
  const actorList = derived<string[]>(() => parseJsonArray<string>(ctx.actors()))

  // The five filter surfaces share one emit shape. Each change handler
  // calls into this with its own `kind` so the host sees a uniform
  // `ship-artifact-filter { kind, value }` stream.
  function emitFilter(kind: FilterKind, e: Event): void {
    const ev = e as { detail?: { value?: string }; target?: { value?: string } }
    const value = ev.detail?.value ?? ev.target?.value ?? ''
    host?.dispatchEvent(
      new CustomEvent('ship-artifact-filter', {
        detail: { kind, value },
        bubbles: true,
        composed: true,
      }),
    )
  }

  return {
    collectionRows, actorList,
    onSearch: (e: Event) => emitFilter('search', e),
    onMime: (e: Event) => emitFilter('mime', e),
    onCollection: (e: Event) => emitFilter('collection', e),
    onActor: (e: Event) => emitFilter('actor', e),
    onAge: (e: Event) => emitFilter('age', e),
  }
})
</controller>

<!--
  <ship-artifact-filters> — @ship/artifacts · the filter bar above the list.

  Presentational. Composes <input> + four <select>s. Reads default
  option sets from `collections` and `actors` (JSON-encoded).
  Emits `ship-artifact-filter { kind, value }` on any change. The host
  wires the events to a list reload (or future Watch filters).

  No SDK. The controller's only job is event emission.

  Props:
    collections   JSON of CollectionRow[]    default '[]'
    actors        JSON of string[]           default '[]'
    disabled      reflect                    default false
-->

<row gap="2">
  <input type="search" size="sm" placeholder="Search by name, id, actor…"
    aria-label="Search artifacts"
    disabled={disabled()}
    data-test="filter-search"
    on:input={onSearch}>

  <select size="sm" aria-label="Filter by type"
    disabled={disabled()}
    data-test="filter-mime"
    on:change={onMime}>
    <option value="">All types</option>
    <option value="image">Images</option>
    <option value="text">Text</option>
    <option value="json">JSON</option>
  </select>

  <select size="sm" aria-label="Filter by collection"
    disabled={disabled()}
    data-test="filter-collection"
    on:change={onCollection}>
    <option value="">All collections</option>
    <for {collectionRows() as c} key={c.id}>
      <option value={c.id}>{c.name || c.id}</option>
    </for>
  </select>

  <select size="sm" aria-label="Filter by actor"
    disabled={disabled()}
    data-test="filter-actor"
    on:change={onActor}>
    <option value="">All actors</option>
    <for {actorList() as a} key={a}>
      <option value={a}>{a}</option>
    </for>
  </select>

  <select size="sm" aria-label="Filter by age"
    disabled={disabled()}
    data-test="filter-age"
    on:change={onAge}>
    <option value="">Any age</option>
    <option value="24h">Last 24h</option>
    <option value="7d">Last 7 days</option>
    <option value="30d">Last 30 days</option>
  </select>
</row>
