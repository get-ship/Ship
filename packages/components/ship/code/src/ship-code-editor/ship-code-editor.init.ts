/**
 * CodeMirror 6 engine for <ship-code-editor>.
 *
 * Lives in a .ts module so the .sail controller stays on the integrate
 * Pure/Action surface (no DOM names, no dynamic import()). Behaviour
 * matches the previous in-controller mount body.
 */

import { effect } from '@ship/sail-runtime'
import type { Signal } from '@ship/sail-runtime'
import { EditorState, Compartment } from '@codemirror/state'
import type { Extension } from '@codemirror/state'
import {
  EditorView, keymap,
  placeholder as cmPlaceholder, lineNumbers,
  highlightActiveLine, highlightActiveLineGutter,
  drawSelection, rectangularSelection,
} from '@codemirror/view'
import {
  defaultHighlightStyle, syntaxHighlighting,
  indentOnInput, bracketMatching, foldGutter, foldKeymap,
} from '@codemirror/language'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  closeBrackets, closeBracketsKeymap,
  autocompletion,
  type CompletionContext, type CompletionResult, type Completion,
} from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { lintGutter, setDiagnostics as cmSetDiagnostics, type Diagnostic } from '@codemirror/lint'
import { suiEditorTheme, suiHighlighting } from './ship-code-editor.theme'
import { loadLanguage } from './ship-code-editor.lang'

export interface SuiDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

export interface SuiCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

export interface EditorSignals {
  value: Signal<string>
  language: Signal<string>
  readonly: Signal<boolean>
  lineNumbers: Signal<boolean>
  placeholder: Signal<string>
}

const DEBOUNCE_MS = 150

export function attachCodeEditor(
  parent: object,
  host: object,
  sigs: EditorSignals,
): { destroy: () => void } {
  const mountParent = parent as HTMLElement
  const ce = host as HTMLElement
  const langCompartment = new Compartment()
  const readonlyCompartment = new Compartment()
  const lineNumCompartment = new Compartment()
  const placeholderCompartment = new Compartment()
  const completionCompartment = new Compartment()

  let view: EditorView | null = null
  let debounceTimer: ReturnType<typeof setTimeout> | null = null
  let suppressUpdate = false
  let pendingDiagnostics: SuiDiagnostic[] | null = null
  let completions: SuiCompletionSource[] = []

  const valSig = sigs.value
  const langSig = sigs.language
  const roSig = sigs.readonly
  const lnSig = sigs.lineNumbers
  const phSig = sigs.placeholder

  function buildCompletionExt(): Extension {
    if (completions.length === 0) return []
    return autocompletion({ override: completions.map(makeCompletionSource), defaultKeymap: true })
  }

  function applyDiagnostics(diagnostics: SuiDiagnostic[]): void {
    if (!view) return
    const st = view.state
    const doc = st.doc
    const cmDiags: Diagnostic[] = diagnostics.map((d) => {
      const line = Math.max(1, Math.min(doc.lines, d.line | 0))
      const startLine = doc.line(line)
      const col = Math.max(1, (d.column ?? 1) | 0)
      const from = Math.min(startLine.to, startLine.from + col - 1)
      const endL = d.endLine ?? line
      const endLineObj = doc.line(Math.max(1, Math.min(doc.lines, endL | 0)))
      const endCol = d.endColumn
      const to = endCol != null
        ? Math.min(endLineObj.to, endLineObj.from + (endCol - 1))
        : endLineObj.to
      return {
        from,
        to: Math.max(from + 1, to),
        severity: d.severity,
        message: d.message,
        source: d.source,
      } as Diagnostic
    })
    view.dispatch(cmSetDiagnostics(st, cmDiags))
  }

  function onDocChange(newValue: string): void {
    suppressUpdate = true
    valSig.set(newValue)
    suppressUpdate = false
    ce.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      ce.dispatchEvent(new CustomEvent('sail-change', {
        detail: { value: newValue }, bubbles: true, composed: true,
      }))
    }, DEBOUNCE_MS)
  }

  const showLn = lnSig() !== false
  const extensions: Extension[] = [
    showLn
      ? lineNumCompartment.of([lineNumbers(), highlightActiveLineGutter(), foldGutter()])
      : lineNumCompartment.of([]),
    history(),
    drawSelection(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    bracketMatching(),
    closeBrackets(),
    rectangularSelection(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    keymap.of([
      {
        key: 'Mod-s',
        preventDefault: true,
        run: () => {
          ce.dispatchEvent(new CustomEvent('sail-save', {
            detail: { value: view?.state.doc.toString() ?? '' },
            bubbles: true, composed: true,
          }))
          return true
        },
      },
      {
        key: 'Escape',
        run: () => {
          ce.dispatchEvent(new CustomEvent('sail-cancel', { bubbles: true, composed: true }))
          return false
        },
      },
      ...closeBracketsKeymap, ...defaultKeymap,
      ...searchKeymap, ...historyKeymap, ...foldKeymap,
      indentWithTab,
    ]),
    suiEditorTheme,
    suiHighlighting,
    lintGutter(),
    langCompartment.of([]),
    completionCompartment.of(buildCompletionExt()),
    readonlyCompartment.of(EditorState.readOnly.of(roSig())),
    placeholderCompartment.of(phSig() ? cmPlaceholder(phSig()) : []),
    EditorView.updateListener.of((update) => {
      if (update.docChanged && !suppressUpdate) {
        onDocChange(update.state.doc.toString())
      }
    }),
  ]

  view = new EditorView({
    state: EditorState.create({ doc: valSig() || '', extensions }),
    parent: mountParent,
  })
  queueMicrotask(() => view?.requestMeasure())

  if (pendingDiagnostics) {
    applyDiagnostics(pendingDiagnostics)
    pendingDiagnostics = null
  }

  const el = ce as HTMLElement & {
    setDiagnostics?: ((d: SuiDiagnostic[]) => void) | undefined
    setCompletions?: ((src: SuiCompletionSource[]) => void) | undefined
    focus?: () => void
  }
  el.setDiagnostics = (diags: SuiDiagnostic[]) => {
    if (!view) { pendingDiagnostics = diags; return }
    applyDiagnostics(diags)
  }
  el.setCompletions = (src: SuiCompletionSource[]) => {
    completions = src ?? []
    view?.dispatch({ effects: completionCompartment.reconfigure(buildCompletionExt()) })
  }
  el.focus = () => { view?.focus() }

  Object.defineProperty(ce, 'value', {
    get: () => view?.state.doc.toString() ?? valSig(),
    set: (v: string) => { valSig.set(v ?? '') },
    configurable: true,
    enumerable: false,
  })

  const disposers: Array<() => void> = []

  disposers.push(effect(() => {
    const newVal = valSig()
    if (!view || suppressUpdate) return
    const current = view.state.doc.toString()
    if (newVal !== current) {
      suppressUpdate = true
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: newVal || '' } })
      suppressUpdate = false
    }
  }))

  disposers.push(effect(() => {
    const lang = langSig()
    if (!view) return
    if (lang) {
      loadLanguage(lang)
        .then((ext) => view?.dispatch({ effects: langCompartment.reconfigure(ext) }))
        .catch(() => {})
    } else {
      view.dispatch({ effects: langCompartment.reconfigure([]) })
    }
  }))

  // ── Browser LSP integration ───────────────────────────────────
  // For `.sui` content, run sui-lsp-browser's pure analyser on the
  // main thread (no worker — the `?worker` Vite suffix isn't supported
  // by the deck bundler, our chrome bundler). Diagnostics stream into
  // setDiagnostics on every debounced doc change; completions are
  // seeded once from allSources. Hover deferred to a follow-up; if
  // the parse cost ever shows in the input loop, switch to a worker
  // with a Bun-compatible URL.
  type LspDiagnoseFn = (text: string) => SuiDiagnostic[]
  type LspAllSourcesFn = () => SuiCompletionSource[]
  let lspDiagnose: LspDiagnoseFn | null = null
  let lspDebounce: ReturnType<typeof setTimeout> | null = null
  let lspInit: Promise<void> | null = null
  async function ensureLsp(): Promise<void> {
    if (lspDiagnose || lspInit) { await lspInit; return }
    lspInit = (async () => {
      try {
        const [diag, comp] = await Promise.all([
          import('@ship/sail-lsp-browser/diagnostics'),
          import('@ship/sail-lsp-browser/completions'),
        ])
        lspDiagnose = (diag as { diagnose: LspDiagnoseFn }).diagnose
        try { el.setCompletions?.((comp as { allSources: LspAllSourcesFn }).allSources()) } catch { /* best-effort */ }
      } catch (err) {
        console.warn('[ship-code-editor] LSP init failed; diagnostics unavailable', err)
      }
    })()
    await lspInit
  }
  disposers.push(effect(() => {
    const lang = langSig()
    if (lang !== 'sui') return
    void ensureLsp()
  }))
  disposers.push(effect(() => {
    const text = valSig()
    if (langSig() !== 'sui') return
    if (lspDebounce) clearTimeout(lspDebounce)
    lspDebounce = setTimeout(async () => {
      if (!lspDiagnose) await ensureLsp()
      if (!lspDiagnose) return
      try { el.setDiagnostics?.(lspDiagnose(text)) } catch { /* best-effort */ }
    }, 350)
  }))
  disposers.push(() => {
    if (lspDebounce) clearTimeout(lspDebounce)
    lspDiagnose = null
    lspInit = null
  })

  disposers.push(effect(() => {
    view?.dispatch({ effects: readonlyCompartment.reconfigure(EditorState.readOnly.of(roSig())) })
  }))

  disposers.push(effect(() => {
    const show = lnSig() !== false
    view?.dispatch({
      effects: lineNumCompartment.reconfigure(
        show ? [lineNumbers(), highlightActiveLineGutter(), foldGutter()] : [],
      ),
    })
  }))

  disposers.push(effect(() => {
    const ph = phSig()
    view?.dispatch({ effects: placeholderCompartment.reconfigure(ph ? cmPlaceholder(ph) : []) })
  }))

  return {
    destroy: () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      for (const d of disposers) d()
      el.setDiagnostics = undefined
      el.setCompletions = undefined
      // Leave el.focus — shadows native HTMLElement.focus; nulling breaks it.
      try {
        Reflect.deleteProperty(ce, 'value')
      } catch {
        ;(ce as { value?: string }).value = undefined
      }
      try { view?.destroy() } catch { /* teardown race */ }
      view = null
    },
  }
}

// CodeMirror's Completion treats absent detail/info as "none"; under
// exactOptionalPropertyTypes we can't pass `undefined` for them, so omit.
function toCmCompletion(it: SuiCompletionSource['items'][number]): Completion {
  const c: Completion = { label: it.label, apply: it.insert ?? it.label }
  if (it.detail !== undefined) c.detail = it.detail
  if (it.info !== undefined) c.info = it.info
  return c
}

function makeCompletionSource(src: SuiCompletionSource) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const { pos, state } = ctx
    const trig = src.trigger
    const start = pos - trig.length
    if (start < 0) return null
    const preceding = state.sliceDoc(start, pos)
    if (preceding !== trig) {
      const word = ctx.matchBefore(/[A-Za-z0-9_.]+/)
      if (!word) return null
      const before = state.sliceDoc(Math.max(0, word.from - trig.length), word.from)
      if (before !== trig) return null
      return {
        from: word.from,
        options: src.items.map(toCmCompletion),
      }
    }
    return {
      from: pos,
      options: src.items.map(toCmCompletion),
    }
  }
}
