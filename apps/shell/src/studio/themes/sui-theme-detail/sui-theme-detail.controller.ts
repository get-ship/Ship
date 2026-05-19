import { controller, signal, ref, effect, chrome, route } from '@ship/sui-runtime'
import type { Scope } from './sui-theme-detail.sui'
import { getPack, isValidTheme, type ThemeId } from '../theme-registry'
import { CATEGORIES, TOKEN_NAMES, type TokenType } from '../token-catalog'
import { snapshotTheme, applyTokens, type TokenMap } from '../theme-snapshot'
import { toCss, toJson, parseImport, downloadFile } from '../theme-io'

// Lazy global route — deferred past module eval (SSR-safe). The detail
// page's theme id is the `:sub` segment of `/studio/themes/<id>`.
let _route: ReturnType<typeof route> | null = null
const rt = (): ReturnType<typeof route> => (_route ??= route())
const currentId = (): ThemeId => {
  const sub = rt().params.sub()
  return isValidTheme(sub) ? sub : 'ship-light'
}

export const TABS: ReadonlyArray<{ id: string; label: string }> = [
  { id: 'all', label: 'All' },
  ...CATEGORIES.map((c) => ({ id: c.id, label: c.label })),
]

export interface EditorRow {
  name: string
  type: TokenType
  value: string
  isColor: boolean
  isShadow: boolean
  dot: boolean
}
export interface EditorGroup { id: string; label: string; rows: EditorRow[] }

// Apply the resolved theme (base snapshot + user overrides) as inline
// custom properties on the preview frame. Inherited custom properties
// cross the shadow boundary into the nested `<sui-*>` preview elements.
function applyPreview(el: HTMLElement | null, base: TokenMap, overrides: TokenMap): void {
  if (!el) return
  el.style.cssText = ''
  applyTokens(el, base)
  applyTokens(el, overrides)
}

export default controller<Scope>({
  scope: () => {
    const overrides = signal<TokenMap>({})
    const activeTab = signal<string>('all')
    const status = signal<string>('')
    const base = signal<TokenMap>({})
    const baseline = signal<TokenMap>(
      typeof document !== 'undefined' ? snapshotTheme('ship-light') : {},
    )
    const previewEl = ref<HTMLElement>(null)
    const fileEl = ref<HTMLInputElement>(null)

    // Re-snapshot the theme + clear overrides whenever the URL theme id
    // changes (navigating between two detail pages keeps this element).
    effect(() => {
      const id = currentId()
      if (typeof document === 'undefined') return
      base.set(snapshotTheme(id))
      overrides.set({})
    })
    // Reflect every token change into the live preview frame.
    effect(() => applyPreview(previewEl(), base(), overrides()))

    const effective = (): TokenMap => {
      const ov = overrides()
      const b = base()
      const out: TokenMap = {}
      for (const n of TOKEN_NAMES) out[n] = ov[n] ?? b[n] ?? ''
      return out
    }

    const flash = (msg: string): void => {
      status.set(msg)
      setTimeout(() => { if (status() === msg) status.set('') }, 1800)
    }

    return {
      overrides, activeTab, status, base, baseline, previewEl, fileEl,

      setTab: (id: string) => activeTab.set(id),
      back: () => chrome.navigate('/studio/themes'),

      edit: (name: string, e: Event) => {
        const v = (e.target as HTMLInputElement).value
        overrides.set({ ...overrides(), [name]: v })
      },
      reset: () => { overrides.set({}); flash('Reset to defaults.') },

      applyToProject: () => {
        const id = currentId()
        if (typeof window !== 'undefined') {
          window.dispatchEvent(new CustomEvent('sui-studio-theme-apply', { detail: { id } }))
        }
        flash(`Applied ${id} to project.`)
      },

      exportCss: () => downloadFile(`theme-${currentId()}.css`, toCss(currentId(), effective()), 'text/css'),
      exportJson: () => downloadFile(`theme-${currentId()}.json`, toJson(currentId(), effective()), 'application/json'),
      triggerImport: () => fileEl()?.click(),
      onImportFile: async (e: Event) => {
        const input = e.target as HTMLInputElement
        const file = input.files?.[0]
        if (!file) return
        const parsed = parseImport(await file.text())
        input.value = ''
        if (!parsed) { flash('Import failed: no tokens found.'); return }
        overrides.set({ ...overrides(), ...parsed })
        flash(`Imported ${Object.keys(parsed).length} tokens.`)
      },
    }
  },

  derived: {
    themeId: () => currentId(),
    name: () => currentId(),
    kind: () => getPack(currentId())?.kind ?? 'light',
    blurb: () => getPack(currentId())?.blurb ?? '',
    metaSuffix: () => getPack(currentId())?.metaSuffix ?? '',
    tabs: () => TABS,
    bg: ({ base }) => base()['--background'] ?? '',
    fg: ({ base }) => base()['--foreground'] ?? '',
    pr: ({ base }) => base()['--primary'] ?? '',
    ac: ({ base }) => base()['--accent'] ?? '',

    overrideCount: ({ base, overrides, baseline }) => {
      const ov = overrides(); const b = base(); const bl = baseline()
      let n = 0
      for (const name of TOKEN_NAMES) {
        const v = (ov[name] ?? b[name] ?? '').trim()
        if (v && v !== (bl[name] ?? '').trim()) n += 1
      }
      return n
    },

    groups: ({ base, overrides, baseline, activeTab }): EditorGroup[] => {
      const ov = overrides(); const b = base(); const bl = baseline(); const tab = activeTab()
      const cats = tab === 'all' ? CATEGORIES : CATEGORIES.filter((c) => c.id === tab)
      return cats.map((c) => ({
        id: c.id,
        label: c.label,
        rows: c.tokens.map((t) => {
          const value = ov[t.name] ?? b[t.name] ?? ''
          const baseV = (bl[t.name] ?? '').trim()
          return {
            name: t.name,
            type: t.type,
            value,
            isColor: t.type === 'color',
            isShadow: t.type === 'shadow',
            dot: !!value.trim() && value.trim() !== baseV,
          }
        }),
      }))
    },
  },
})
