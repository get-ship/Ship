import { controller, signal, ref, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-va-source.sui'

interface CodeEditor extends HTMLElement {
  value: string
  setDiagnostics?: (diags: Array<{ line: number; severity: 'error' | 'warning'; message: string }>) => void
}

interface ViewMcp {
  readSource(name: string): Promise<{ source: string }>
  writeSource(name: string, source: string): Promise<{ ok: true }>
}

declare global {
  interface Window {
    __viewMcp?: ViewMcp
  }
}

export default controller<Scope>({
  scope: (state) => {
    const source = signal<string>('')
    const dirty = signal<boolean>(false)
    const saved = signal<boolean>(false)
    const editorEl = ref<CodeEditor>(null)
    const s = state as unknown as { view: () => string }

    function onChange(e: Event): void {
      const ce = e as CustomEvent<{ value?: string }>
      const next = ce.detail?.value ?? (editorEl()?.value ?? '')
      source.set(next)
      dirty.set(true)
      saved.set(false)
    }

    async function onSave(): Promise<void> {
      const name = s.view()
      if (!name) return
      const mcp = window.__viewMcp
      if (!mcp) return
      const value = editorEl()?.value ?? source()
      try {
        await mcp.writeSource(name, value)
        source.set(value)
        dirty.set(false)
        saved.set(true)
        window.setTimeout(() => saved.set(false), 1500)
      } catch { /* surface via chrome.notify elsewhere if needed */ }
    }

    return { source, dirty, saved, editorEl, onChange, onSave }
  },

  mount(state, host) {
    const s = state as unknown as {
      source: ReturnType<typeof signal<string>>
      dirty: ReturnType<typeof signal<boolean>>
      saved: ReturnType<typeof signal<boolean>>
      editorEl: () => CodeEditor | null
      view: () => string
    }

    let loadedFor: string | null = null

    async function loadSource(name: string): Promise<void> {
      if (!name) return
      if (loadedFor === name) return
      loadedFor = name
      const mcp = window.__viewMcp
      if (!mcp) return
      try {
        const res = await mcp.readSource(name)
        s.source.set(res.source)
        s.dirty.set(false)
        s.saved.set(false)
      } catch { /* best-effort */ }
    }

    const stopLoadEffect = effect(() => {
      const v = s.view()
      if (v) void loadSource(v)
    })

    // Compile-error → editor diagnostics bridge.
    function onErrored(e: Event): void {
      const detail = (e as CustomEvent).detail as { view?: string; errors?: Array<{ line: number; message: string; severity?: 'error' | 'warning' }> } | undefined
      if (!detail || detail.view !== s.view()) return
      const ed = s.editorEl()
      if (!ed?.setDiagnostics) return
      ed.setDiagnostics((detail.errors ?? []).map((e2) => ({
        line: e2.line, message: e2.message, severity: e2.severity ?? 'error',
      })))
    }
    document.addEventListener('sui:view:errored', onErrored)

    // Imperative API used by tab activation: re-load when activated.
    Object.defineProperty(host, 'reload', {
      configurable: true,
      value: () => { loadedFor = null; return loadSource(s.view()) },
    })

    return () => {
      stopLoadEffect()
      document.removeEventListener('sui:view:errored', onErrored)
    }
  },
})
