import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
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

export default controller<Scope>({
  scope: ({ ship, tokenCatalog }) => {
    const draftTokens = signal<Record<string, string>>({ ...DRAFT_DEFAULTS })

    const tokenEntries = derived(() => {
      const catalog = unwrap(tokenCatalog)
      if (!catalog?.tokens) {
        return Object.entries(DRAFT_DEFAULTS).map(([name, def]) => ({
          name,
          meta: { type: 'color' as const, default: def, intent: undefined as string | undefined },
        }))
      }
      return Object.entries(catalog.tokens).map(([name, meta]) => ({ name, meta }))
    })

    const draftStyle = derived(() =>
      Object.entries(draftTokens())
        .map(([k, v]) => `--draft-${k}: ${v}`)
        .join('; ')
    )

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const contrastReport = live(() => s.themes.contrast({ tokens: draftTokens() }))

    const contrastRows = derived(() => {
      const env = contrastReport()
      if (env.kind !== 'ok') return []
      const pairs = (env.value as { pairs?: unknown[] })?.pairs ?? []
      return (pairs as Array<Record<string, unknown>>).map((p) => ({
        ...p,
        ratioStr: typeof p.ratio === 'number' ? (p.ratio as number).toFixed(2) : '—',
        minStr: typeof p.required_min === 'number' ? (p.required_min as number).toFixed(2) : '—',
      }))
    })

    return { draftTokens, tokenEntries, draftStyle, contrastReport, contrastRows }
  },
})
