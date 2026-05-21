// Catalog loader for the Components Studio section. Single source of
// truth is the daemon's published catalog (`/api/sui/catalog/published`,
// emitted by @ship/sui-catalog from each primitive's CEM). We project it
// to the shape the list + detail surfaces render, and join the editorial
// category/blurb layer from component-meta.ts.

import { CATEGORIES, metaFor, type CategoryId } from './component-meta'

export interface PropEntry {
  name: string
  type: string
  default?: string
  description?: string
  /** Closed set of allowed string values for a union prop. Drives the
   *  segmented knob + variant×tone matrix axes. */
  values?: string[]
}

export interface SlotEntry { name: string; description?: string }

export interface ComponentEntry {
  /** Tag name, e.g. `sui-button`. Doubles as the route id. */
  tag: string
  category: CategoryId
  /** One-line blurb — catalog description if present, else curated. */
  description: string
  props: PropEntry[]
  slots: SlotEntry[]
  /** Repo-relative `.sui` source path, from the catalog. */
  sourcePath?: string
  /** Owning package, e.g. `@ship/primitives`. */
  package?: string
}

export interface CategoryGroup {
  id: CategoryId
  label: string
  blurb: string
  items: ComponentEntry[]
  /** Content key (id + member tags). Used as the template `{#for}` key so
   *  a group whose membership shrinks under a filter rebuilds its row list
   *  instead of leaving stale rows — the nested keyed `{#for}` reconciles
   *  the group list but not a *reused* group's inner item list. */
  key: string
}

// ── Wire shapes (subset of the published catalog) ────────────────────
interface WireAttr {
  name: string
  type?: { text?: string }
  default?: string
  description?: string
  values?: unknown
}
interface WireSlot { name?: string; description?: string }
interface WireEntry {
  tagName: string
  description?: string
  firstDesc?: string
  attributes?: WireAttr[]
  slots?: WireSlot[]
  sourcePath?: string
  package?: string
}
interface WireCatalog { entries?: WireEntry[] }

// Build objects with only the keys that are actually present —
// `exactOptionalPropertyTypes` rejects assigning `undefined` to an
// optional field, so we conditionally attach the optional props.
function toProp(a: WireAttr): PropEntry {
  const p: PropEntry = { name: a.name, type: a.type?.text ?? 'string' }
  if (a.default !== undefined) p.default = a.default
  if (a.description !== undefined) p.description = a.description
  if (Array.isArray(a.values)) p.values = a.values as string[]
  return p
}

function toSlot(s: WireSlot & { name: string }): SlotEntry {
  const out: SlotEntry = { name: s.name }
  if (s.description !== undefined) out.description = s.description
  return out
}

function toEntry(e: WireEntry): ComponentEntry {
  const meta = metaFor(e.tagName)
  const catalogDesc = (e.firstDesc || e.description || '').trim()
  const out: ComponentEntry = {
    tag: e.tagName,
    category: meta.category,
    description: catalogDesc || meta.blurb,
    props: (e.attributes ?? []).map(toProp),
    slots: (e.slots ?? [])
      .filter((s): s is WireSlot & { name: string } => !!s.name)
      .map(toSlot),
  }
  if (e.sourcePath !== undefined) out.sourcePath = e.sourcePath
  if (e.package !== undefined) out.package = e.package
  return out
}

let _cache: Promise<ComponentEntry[]> | null = null

/** Fetch + normalize the published catalog. Cached per module so the
 *  list and detail surfaces share one round-trip. Empty array on any
 *  failure — the surfaces render their own empty state rather than
 *  throwing. */
export function loadComponents(): Promise<ComponentEntry[]> {
  if (_cache) return _cache
  _cache = (async () => {
    try {
      const res = await fetch('/api/sui/catalog/published', { credentials: 'same-origin' })
      if (!res.ok) return []
      const doc = (await res.json()) as WireCatalog
      return (doc.entries ?? [])
        .filter((e) => !!e.tagName)
        .map(toEntry)
        .sort((a, b) => a.tag.localeCompare(b.tag))
    } catch {
      return []
    }
  })()
  return _cache
}

/** Group entries into the ordered category list. Categories with no
 *  members are dropped so the index shows no empty headers. */
export function groupByCategory(entries: readonly ComponentEntry[]): CategoryGroup[] {
  const byCat = new Map<CategoryId, ComponentEntry[]>()
  for (const e of entries) {
    const list = byCat.get(e.category) ?? []
    list.push(e)
    byCat.set(e.category, list)
  }
  const groups: CategoryGroup[] = []
  for (const c of CATEGORIES) {
    const items = byCat.get(c.id)
    if (!items || items.length === 0) continue
    const key = `${c.id}#${items.map((e) => e.tag).join(',')}`
    groups.push({ id: c.id, label: c.label, blurb: c.blurb, items, key })
  }
  return groups
}

export function findEntry(
  entries: readonly ComponentEntry[],
  tag: string,
): ComponentEntry | null {
  return entries.find((e) => e.tag === tag) ?? null
}
