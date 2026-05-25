// Shared catalog loader for ship-ui-studio.
// Fetches three sources: Lit CEM, SUI registry, daemon views.

import type { ComponentExample } from '@ship/sui-runtime'

export interface CatalogEntry {
  id: string
  tag: string
  kind: 'lit' | 'sui' | 'view'
  description?: string
  props?: PropEntry[]
  /** Co-located showcase scenarios, read from the component's CEM declaration. */
  examples?: ComponentExample[]
}

export interface PropEntry {
  name: string
  type: string
  default?: string
  description?: string
  /// Closed set of allowed string values, when the prop is a union. Used
  /// by the studio prop panel to render a `<select>` instead of a free
  /// text input.
  values?: string[]
}

// The SUI catalog (`@ship/sui-catalog`, served at /api/sui/catalog/published)
// is the single source of truth for installed-pack components: tag, props,
// examples, and routing. There is no Lit path and no per-tag hide list — the
// catalog already excludes shell-local chrome and view-local components.
async function loadSuiComponents(): Promise<CatalogEntry[]> {
  const byTag = await loadSuiCatalog()
  return [...byTag.keys()]
    .sort()
    .map(tag => {
      const e = byTag.get(tag)!
      return {
        id: tag,
        tag,
        kind: 'sui' as const,
        props: e.props,
        ...(e.description ? { description: e.description } : {}),
        ...(e.examples?.length ? { examples: e.examples } : {}),
      }
    })
}

// One wire entry from the published catalog. Carries the standard CEM fields
// the studio needs plus the co-located `examples` (fun fixtures).
interface CatalogEntryWire {
  tagName: string
  description?: string
  attributes?: Array<{
    name: string
    type?: { text?: string }
    default?: string
    description?: string
    values?: unknown
  }>
  examples?: ComponentExample[]
}
interface SuiCatalogEntry {
  props: PropEntry[]
  examples?: ComponentExample[]
  description?: string
}

async function loadSuiCatalog(): Promise<Map<string, SuiCatalogEntry>> {
  const out = new Map<string, SuiCatalogEntry>()
  try {
    const res = await fetch('/api/sui/catalog/published', { credentials: 'same-origin' })
    if (!res.ok) return out
    const doc = await res.json() as { entries?: CatalogEntryWire[] }
    for (const e of doc.entries ?? []) {
      if (!e.tagName) continue
      out.set(e.tagName, {
        props: (e.attributes ?? []).map(a => ({
          name: a.name,
          type: a.type?.text ?? 'string',
          default: a.default,
          description: a.description,
          values: Array.isArray(a.values) ? a.values as string[] : undefined,
        })),
        examples: e.examples,
        description: e.description,
      })
    }
    return out
  } catch {
    return out
  }
}

// The components explorer shows installed-pack components — sourced entirely
// from the SUI catalog. Views are not components; they live in the view-picker.
export async function loadCatalog(): Promise<CatalogEntry[]> {
  return loadSuiComponents()
}
