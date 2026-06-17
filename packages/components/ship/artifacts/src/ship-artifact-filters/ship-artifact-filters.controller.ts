// <ship-artifact-filters> controller — parses the JSON props into the
// derived row arrays the template binds, and owns the
// `ship-artifact-filter { kind, value }` event surface. No SDK, no
// live data.

import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-artifact-filters.sui'

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

export default controller({
  scope: ({ collections, actors }: ViewScope<Scope>, host) => {
    const collectionRows = derived<CollectionRow[]>(() => parseJsonArray<CollectionRow>(collections()))
    const actorList = derived<string[]>(() => parseJsonArray<string>(actors()))

    // The five filter surfaces share one emit shape. Each on:sui-* handler
    // calls into this with its own `kind` so the host sees a uniform
    // `ship-artifact-filter { kind, value }` stream.
    function emitFilter(kind: FilterKind, e: Event): void {
      const value = (e as CustomEvent<{ value: string }>).detail?.value ?? ''
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
  },
})
