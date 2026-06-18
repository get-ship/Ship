// <ship-artifact-preview> controller — derives the preview kind from
// the record's mime and, for the text kind, fetches the body in mount.
//
// Why a controller at all: image / html / binary previews are zero-fetch
// (browsers do the work). Text needs a fetch — the legacy view's
// `loadPreview()` lives here, scoped to the host-supplied URL.
//
// The fetch is routed through the SDK's typed `ship.artifacts.get({ id, as: 'text' })`
// seam so the runtime's SES compartment enforces the policy gate. The
// mount fires the fetch and writes the result into a signal; the renderable
// expression subset forbids async work outside `on:` handlers and lifecycle.

import { controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import { previewKindFor } from '../decorate'
import type { Scope } from './ship-artifact-preview.sui'

interface RecordProp {
  id: string
  name: string
  mime: string
}

function parseRecord(raw: string): RecordProp {
  try {
    const o = JSON.parse(raw) as Partial<RecordProp>
    return { id: o.id ?? '', name: o.name ?? '', mime: o.mime ?? '' }
  } catch {
    return { id: '', name: '', mime: '' }
  }
}

const TEXT_CAP = 50_000

export default controller({
  scope: ({ record }: ViewScope<Scope>) => {
    const textBody = signal<string>('')
    const textLoading = signal<boolean>(false)
    const textError = signal<string>('')

    const row = derived<RecordProp>(() => parseRecord(record()))
    const kind = derived<'image' | 'html' | 'text' | 'binary'>(() => previewKindFor(row().mime))
    const altText = derived<string>(() => row().name || row().id)
    const downloadName = derived<string>(() => row().name || row().id || 'artifact')

    return { textBody, textLoading, textError, row, kind, altText, downloadName }
  },

  mount(state) {
    // `state.row` is the unwrapped derived value; `state.url` is a host
    // prop. The MutableState proxy reads declared signal keys without `.set`.
    const { row, url } = state as unknown as { row: RecordProp; url: string }
    if (previewKindFor(row.mime) !== 'text') return
    if (!url) return
    if (!row.id) return
    state.textLoading = true
    state.textError = ''
    state.textBody = ''
    ship.artifacts
      .get({ id: row.id, as: 'text' })
      .then((text) => {
        state.textBody = text.length > TEXT_CAP ? text.slice(0, TEXT_CAP) + '\n…(truncated)' : text
      })
      .catch((err: Error) => { state.textError = err.message })
      .finally(() => { state.textLoading = false })
  },
})
