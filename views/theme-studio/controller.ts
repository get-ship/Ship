// Theme studio controller — token catalog comes from a baked map of
// design-token defaults; contrast scoring posts the live draft to the
// daemon's REST endpoint.
//
// Why not the SDK? The runtime-bundled SDK client carries neither a
// `themes` namespace nor a registered `ship.themes.*` op on the
// daemon, so any `<data live source="ship.themes.…" />` declaration
// resolves to a perpetual `kind: 'loading'` (registry returns `null`,
// no SSE patches fire). The daemon does expose `POST /api/v1/themes/
// contrast` for the contrast report; we drive it from a tracked
// `live()` thunk that re-POSTs whenever the draft tokens change.

import { controller, signal, derived, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const DRAFT_DEFAULTS: Record<string, string> = {
  'foreground': '#0a0a0a',
  'background': '#ffffff',
  'primary': 'oklch(0.53 0.22 263)',
  'primary-foreground': '#ffffff',
  'border': '#e5e5e5',
  'muted': '#f5f5f5',
  'muted-foreground': '#737373',
  'destructive': '#dc2626',
  'destructive-foreground': '#ffffff',
  'success': '#16a34a',
  'warning': '#d97706',
  'accent': '#f4f4f5',
  'accent-foreground': '#0a0a0a',
}

interface ContrastPair {
  label?: string
  foreground?: string
  background?: string
  ratio?: number
  required_min?: number
  aa_body?: boolean
  aaa_body?: boolean
  passes?: boolean
}

interface ContrastReport {
  pairs?: ContrastPair[]
}

export default controller<Scope>({
  scope: () => {
    const draftTokens = signal<Record<string, string>>({ ...DRAFT_DEFAULTS })

    const tokenEntries = derived(() =>
      Object.entries(DRAFT_DEFAULTS).map(([name, def]) => ({
        name,
        meta: { type: 'color' as const, default: def, intent: undefined as string | undefined },
      })),
    )

    const draftStyle = derived(() =>
      Object.entries(draftTokens())
        .map(([k, v]) => `--draft-${k}: ${v}`)
        .join('; ')
    )

    const contrastReport = live<ContrastReport>(async () => {
      const tokens = draftTokens()
      const resp = await fetch('/api/v1/themes/contrast', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        credentials: 'same-origin',
        body: JSON.stringify({ tokens }),
      })
      if (!resp.ok) {
        throw new Error(`contrast ${resp.status}: ${await resp.text()}`)
      }
      return (await resp.json()) as ContrastReport
    })

    const contrastRows = derived(() => {
      const env = contrastReport()
      if (env.kind !== 'ok') return []
      const pairs = env.value.pairs ?? []
      return pairs.map((p) => ({
        ...p,
        ratioStr: typeof p.ratio === 'number' ? p.ratio.toFixed(2) : '—',
        minStr: typeof p.required_min === 'number' ? p.required_min.toFixed(2) : '—',
      }))
    })

    return { draftTokens, tokenEntries, draftStyle, contrastReport, contrastRows }
  },
})
