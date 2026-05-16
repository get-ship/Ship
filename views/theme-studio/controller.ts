// Theme Studio controller.
//
// Reads the canonical token catalog (token-catalog.ts), exposes per-layer
// editor rows, and writes live edits through `chrome.theme.set(token, value)`
// — the IPC wrapper persists into the shell's per-view claim registry and
// drops them on unmount. Pack changes drive `applyTheme(id)` which flips
// `data-theme` on <html>, then re-reads each token's resolved value into
// the draft signal so the editor inputs reflect the active pack. User
// overrides ride on top of pack values until "Reset to pack."
//
// Persistence:
//   PUT /api/v1/settings { key: 'theme.overrides', value: <JSON> }
// Round-trip is plain JSON so Export/Import use the same shape.

import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import { applyTheme, readStoredTheme, persistTheme, isValidTheme, type ThemeId } from '../../packages/ui-components/src/tokens/themes'
import type { Scope } from './page.sui'
import { LAYERS, THEME_PACKS, ALL_TOKENS } from './token-catalog'

const STORAGE_KEY = 'theme.overrides'

interface OverridePayload {
  pack: ThemeId
  values: Record<string, string>
}

function readResolved(token: string): string {
  if (typeof window === 'undefined') return ''
  return getComputedStyle(document.documentElement).getPropertyValue(token).trim()
}

function snapshotPack(): Record<string, string> {
  const out: Record<string, string> = {}
  for (const t of ALL_TOKENS) out[t.name] = readResolved(t.name)
  return out
}

async function loadOverrides(): Promise<OverridePayload | null> {
  try {
    const res = await fetch(`/api/v1/settings/${encodeURIComponent(STORAGE_KEY)}`, {
      credentials: 'same-origin',
    })
    if (!res.ok) return null
    const body = (await res.json()) as { data?: { value?: unknown } } | { value?: unknown }
    const raw = 'data' in body ? body.data?.value : (body as { value?: unknown }).value
    if (typeof raw !== 'string') return null
    const parsed = JSON.parse(raw) as Partial<OverridePayload>
    if (!parsed || typeof parsed !== 'object' || !parsed.values) return null
    const pack = isValidTheme(parsed.pack) ? parsed.pack : readStoredTheme()
    return { pack, values: parsed.values as Record<string, string> }
  } catch {
    return null
  }
}

async function saveOverrides(payload: OverridePayload): Promise<void> {
  await fetch('/api/v1/settings', {
    method: 'PUT',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ key: STORAGE_KEY, value: JSON.stringify(payload) }),
  })
}

export default controller<Scope>({
  scope: () => {
    const activePack = signal<ThemeId>(readStoredTheme())
    const packDefaults = signal<Record<string, string>>({})
    const overrides = signal<Record<string, string>>({})
    const savedSnapshot = signal<OverridePayload | null>(null)
    const status = signal<{ tone: 'idle' | 'saved' | 'error'; msg: string }>({ tone: 'idle', msg: '' })

    const layers = derived(() => LAYERS)
    const packs = derived(() => THEME_PACKS)

    // Resolved-value lookup table for inputs: override wins, then pack
    // default, then the computed style (covers tokens we haven't snapshotted
    // yet). Re-derives whenever overrides or pack-defaults change. Templates
    // read via `values()[tok.name]` — sui-compiler rejects parameterised
    // function calls inside expressions, so we materialise the whole map.
    const values = derived<Record<string, string>>(() => {
      const o = overrides()
      const d = packDefaults()
      const out: Record<string, string> = {}
      for (const t of ALL_TOKENS) {
        if (t.name in o) out[t.name] = o[t.name] ?? ''
        else if (t.name in d) out[t.name] = d[t.name] ?? ''
        else out[t.name] = readResolved(t.name)
      }
      return out
    })

    function setToken(token: string, value: string): void {
      const next = { ...overrides(), [token]: value }
      overrides.set(next)
      chrome.theme.set(token, value)
    }

    function applyAllOverrides(values: Record<string, string>): void {
      for (const [k, v] of Object.entries(values)) {
        chrome.theme.set(k, v)
      }
    }

    function clearAllOverrides(values: Record<string, string>): void {
      for (const k of Object.keys(values)) chrome.theme.clear(k)
    }

    function selectPack(idRaw: string): void {
      if (!isValidTheme(idRaw)) return
      const id = idRaw as ThemeId
      clearAllOverrides(overrides())
      const next = applyTheme(id)
      persistTheme(next)
      activePack.set(next)
      overrides.set({})
      // Re-snapshot defaults after data-theme flips. Defer one frame so
      // the cascade settles before getComputedStyle reads.
      requestAnimationFrame(() => packDefaults.set(snapshotPack()))
    }

    function resetToPack(): void {
      clearAllOverrides(overrides())
      overrides.set({})
      status.set({ tone: 'idle', msg: 'Reset to pack defaults.' })
    }

    function resetToSaved(): void {
      const saved = savedSnapshot()
      clearAllOverrides(overrides())
      if (!saved) {
        overrides.set({})
        status.set({ tone: 'idle', msg: 'Reverted (no saved overrides).' })
        return
      }
      overrides.set({ ...saved.values })
      applyAllOverrides(saved.values)
      status.set({ tone: 'idle', msg: 'Reverted to last saved.' })
    }

    async function savePersist(): Promise<void> {
      const payload: OverridePayload = { pack: activePack(), values: { ...overrides() } }
      try {
        await saveOverrides(payload)
        savedSnapshot.set(payload)
        status.set({ tone: 'saved', msg: 'Saved.' })
        setTimeout(() => {
          if (status().tone === 'saved') status.set({ tone: 'idle', msg: '' })
        }, 1600)
      } catch (err) {
        status.set({ tone: 'error', msg: `Save failed: ${(err as Error).message}` })
      }
    }

    function exportJson(): void {
      const payload: OverridePayload = { pack: activePack(), values: { ...overrides() } }
      const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `theme-${payload.pack}-overrides.json`
      a.click()
      URL.revokeObjectURL(url)
    }

    async function importFiles(files: FileList | null): Promise<void> {
      if (!files || files.length === 0) return
      const file = files[0]
      if (!file) return
      try {
        const text = await file.text()
        const parsed = JSON.parse(text) as Partial<OverridePayload>
        if (!parsed.values || typeof parsed.values !== 'object') {
          throw new Error('missing "values" object')
        }
        const pack: ThemeId = isValidTheme(parsed.pack) ? parsed.pack : activePack()
        if (pack !== activePack()) selectPack(pack)
        clearAllOverrides(overrides())
        const values = parsed.values as Record<string, string>
        overrides.set({ ...values })
        applyAllOverrides(values)
        status.set({ tone: 'saved', msg: `Imported ${Object.keys(values).length} overrides.` })
      } catch (err) {
        status.set({ tone: 'error', msg: `Import failed: ${(err as Error).message}` })
      }
    }

    return {
      activePack,
      packDefaults,
      overrides,
      savedSnapshot,
      status,
      layers,
      packs,
      values,
      setToken,
      selectPack,
      resetToPack,
      resetToSaved,
      savePersist,
      exportJson,
      importFiles,
    }
  },

  mount(state: unknown) {
    const scope = state as {
      packDefaults: { set: (v: Record<string, string>) => void }
      overrides: { set: (v: Record<string, string>) => void } & (() => Record<string, string>)
      savedSnapshot: { set: (v: OverridePayload | null) => void }
      activePack: { set: (id: ThemeId) => void } & (() => ThemeId)
    }

    // Initial snapshot of the active pack's computed token values so
    // editor inputs aren't blank on first paint.
    scope.packDefaults.set(snapshotPack())

    // Pull any saved overrides from the daemon and apply them.
    void loadOverrides().then((saved) => {
      if (!saved) return
      scope.savedSnapshot.set(saved)
      if (saved.pack !== scope.activePack()) {
        applyTheme(saved.pack)
        scope.activePack.set(saved.pack)
        requestAnimationFrame(() => scope.packDefaults.set(snapshotPack()))
      }
      scope.overrides.set({ ...saved.values })
      for (const [k, v] of Object.entries(saved.values)) chrome.theme.set(k, v)
    })

    // Re-snapshot pack defaults whenever the document theme changes
    // out-of-band (e.g. via the top-bar theme picker).
    const onThemeChange = () => {
      requestAnimationFrame(() => scope.packDefaults.set(snapshotPack()))
    }
    document.addEventListener('ship:theme:change', onThemeChange)

    return () => {
      document.removeEventListener('ship:theme:change', onThemeChange)
    }
  },
})

export type { TokenDef } from './token-catalog'
