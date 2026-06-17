// <ship-artifact-list> controller — live subscription to
// `ship.artifacts.list({})` plus the default six-column definition.
//
// Template owns the loading / error / ok branches via
// `{#while-loading}` / `{#while-error}` / `{#if artifacts().kind === 'ok'}`.
// The controller only derives data; it does NOT branch on
// `artifacts().kind` (the sui-controller-no-load-state lint).
//
// `decorate()` enriches the raw ArtifactRecord rows with the
// format-helper fields the default column set reads:
//   size_label · id_short · created_short · viewer_label · viewer_url

import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { ArtifactRecord } from '@ship/sdk-client'
import { decorate, type ArtifactRow } from '../decorate'
import type { Scope } from './ship-artifact-list.sui'

interface ColumnDef {
  field: string
  label: string
}

const DEFAULT_COLUMNS: ColumnDef[] = [
  { field: 'name', label: 'Name' },
  { field: 'mime', label: 'Type' },
  { field: 'viewer_label', label: 'Viewer' },
  { field: 'owner_actor', label: 'Actor' },
  { field: 'created_short', label: 'Age' },
  { field: 'size_label', label: 'Size' },
]

const EMPTY = { items: [] as ArtifactRecord[] }

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const artifacts = live(() => ship.artifacts.list({}))
    const rows = derived<ArtifactRow[]>(() => decorate(unwrapOr(artifacts, EMPTY).items))
    const columnDefs = derived<ColumnDef[]>(() => DEFAULT_COLUMNS)
    const totalCount = derived<number>(() => rows().length)
    const visibleCount = derived<number>(() => rows().length)
    return { artifacts, rows, columnDefs, totalCount, visibleCount }
  },
})
