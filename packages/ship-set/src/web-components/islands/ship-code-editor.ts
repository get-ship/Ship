/**
 * <ship-code-editor>
 *
 * CodeMirror 6 wrapped as a lazy-loaded web component island. Language
 * extensions are dynamically imported when the `language` attribute is set,
 * keeping the base bundle lightweight.
 *
 * Attributes:
 *   - value: initial code string
 *   - language: yaml|json|jsonc|javascript|typescript|rust|markdown|html|css|shell|python
 *   - placeholder: shown when empty
 *   - readonly: disable editing
 *   - line-numbers: show line numbers (default true)
 *   - size: sm|md|lg (affects font size + line height)
 *   - tone: focus ring color (default|info|success|warning|destructive)
 *   - theme: light|dark|auto (auto reads .dark class on documentElement)
 *
 * Events:
 *   - ship-change: CustomEvent { detail: { value } } debounced 150ms
 *   - input: native bubbling (for Datastar data-bind)
 *   - ship-save: CustomEvent { detail: { value } } — fired on Mod-s
 *   - ship-cancel: CustomEvent — fired on Escape (does not prevent the
 *     default Escape behavior; search panels etc. still close)
 *
 * Methods:
 *   - setDiagnostics(Array<{line, column, endLine?, endColumn?, severity, message, source?}>):
 *       render CodeMirror lint markers for server-side diagnostics.
 *       line/column are 1-based; endLine/endColumn are optional and default
 *       to end-of-line if omitted. Severity is "error"|"warning"|"info".
 *
 * Form association: ElementInternals, formAssociated.
 */

import { LitElement, html } from 'lit'
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
import {
  defaultKeymap, history, historyKeymap, indentWithTab,
} from '@codemirror/commands'
import {
  closeBrackets, closeBracketsKeymap,
  autocompletion, type CompletionContext, type CompletionResult,
} from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { lintGutter, setDiagnostics, type Diagnostic } from '@codemirror/lint'
import { shipEditorTheme, shipHighlighting } from './ship-code-editor.theme'
import { codeEditorStyles } from './ship-code-editor.styles'
import { loadLanguage } from './ship-code-editor.languages'
import type { SupportedLanguage } from './ship-code-editor.languages'

/** Server-shape diagnostic — 1-based line/column, matches the daemon's
 *  ValidationIssue response for pipelines + other edited configs. */
export interface ShipDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

/** Declarative completion source — the editor converts these into
 *  CodeMirror completion providers so callers don't need to import
 *  `@codemirror/autocomplete` types.
 *
 *  - `trigger` is the literal text that must appear immediately before
 *    the cursor for this source to fire (e.g. `ship.`, `{% `, `{{ `).
 *  - `items` are returned verbatim; `label` is mandatory, `detail` and
 *    `info` are rendered in the CM popup.
 *  - `insert` overrides the inserted text; defaults to `label`. */
export interface ShipCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

const DEBOUNCE_MS = 150

export class ShipCodeEditor extends LitElement {
  static formAssociated = true
  static styles = codeEditorStyles

  static properties = {
    value: { type: String },
    language: { type: String },
    placeholder: { type: String },
    readonly: { type: Boolean, reflect: true },
    'line-numbers': { type: Boolean, attribute: 'line-numbers' },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    theme: { type: String },
  }

  value = ''
  language: SupportedLanguage | '' = ''
  placeholder = ''
  readonly = false
  'line-numbers' = true
  size: 'sm' | 'md' | 'lg' = 'md'
  tone = 'default'
  theme: 'light' | 'dark' | 'auto' = 'auto'

  private _view: EditorView | null = null
  private _debounceTimer: ReturnType<typeof setTimeout> | null = null
  private _internals: ElementInternals | null = null
  private _langCompartment = new Compartment()
  private _readonlyCompartment = new Compartment()
  private _lineNumbersCompartment = new Compartment()
  private _placeholderCompartment = new Compartment()
  private _completionCompartment = new Compartment()
  private _completions: ShipCompletionSource[] = []
  private _suppressUpdate = false
  /** Pending diagnostics queued before the view finishes constructing. */
  private _pendingDiagnostics: ShipDiagnostic[] | null = null

  constructor() {
    super()
    if (typeof ElementInternals !== 'undefined') {
      this._internals = this.attachInternals()
    }
  }

  async firstUpdated(): Promise<void> {
    const langExt = this.language ? await loadLanguage(this.language) : []
    const showLn = this['line-numbers'] !== false

    const extensions: Extension[] = [
      showLn
        ? this._lineNumbersCompartment.of([
            lineNumbers(), highlightActiveLineGutter(), foldGutter(),
          ])
        : this._lineNumbersCompartment.of([]),
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
        // Host-intent bindings — fire `ship-save` / `ship-cancel` so the
        // embedding view can wire them to its own save/exit flow. We
        // keep CodeMirror's default keymap intact by returning true from
        // our handlers (don't let the keystroke fall through).
        {
          key: 'Mod-s',
          preventDefault: true,
          run: () => {
            this.dispatchEvent(new CustomEvent('ship-save', {
              detail: { value: this.value }, bubbles: true, composed: true,
            }))
            return true
          },
        },
        {
          key: 'Escape',
          run: () => {
            this.dispatchEvent(new CustomEvent('ship-cancel', {
              bubbles: true, composed: true,
            }))
            // Return false so CodeMirror's default Escape handlers
            // (close search panel, exit multi-cursor) still run.
            return false
          },
        },
        ...closeBracketsKeymap, ...defaultKeymap,
        ...searchKeymap, ...historyKeymap, ...foldKeymap,
        indentWithTab,
      ]),
      shipEditorTheme,
      shipHighlighting,
      lintGutter(),
      this._langCompartment.of(langExt),
      this._completionCompartment.of(this._buildCompletionExt()),
      this._readonlyCompartment.of(EditorState.readOnly.of(this.readonly)),
      this._placeholderCompartment.of(
        this.placeholder ? cmPlaceholder(this.placeholder) : [],
      ),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && !this._suppressUpdate) {
          this._onDocChange(update.state.doc.toString())
        }
      }),
    ]

    const root = this.shadowRoot!.querySelector('.editor-root')!
    this._view = new EditorView({
      state: EditorState.create({ doc: this.value || '', extensions }),
      parent: root,
    })
    // Flush any diagnostics that arrived before the view was ready.
    if (this._pendingDiagnostics) {
      this._applyDiagnostics(this._pendingDiagnostics)
      this._pendingDiagnostics = null
    }
  }

  async updated(changed: Map<string, unknown>): Promise<void> {
    if (!this._view) return

    if (changed.has('value')) {
      const current = this._view.state.doc.toString()
      if (this.value !== current) {
        this._suppressUpdate = true
        this._view.dispatch({
          changes: { from: 0, to: this._view.state.doc.length, insert: this.value || '' },
        })
        this._suppressUpdate = false
      }
    }

    if (changed.has('language')) {
      const ext = this.language ? await loadLanguage(this.language) : []
      this._view.dispatch({ effects: this._langCompartment.reconfigure(ext) })
    }

    if (changed.has('readonly')) {
      this._view.dispatch({
        effects: this._readonlyCompartment.reconfigure(
          EditorState.readOnly.of(this.readonly),
        ),
      })
    }

    if (changed.has('line-numbers')) {
      const show = this['line-numbers'] !== false
      this._view.dispatch({
        effects: this._lineNumbersCompartment.reconfigure(
          show ? [lineNumbers(), highlightActiveLineGutter(), foldGutter()] : [],
        ),
      })
    }

    if (changed.has('placeholder')) {
      this._view.dispatch({
        effects: this._placeholderCompartment.reconfigure(
          this.placeholder ? cmPlaceholder(this.placeholder) : [],
        ),
      })
    }
  }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this._debounceTimer) clearTimeout(this._debounceTimer)
    if (this._view) {
      this._view.destroy()
      this._view = null
    }
  }

  private _onDocChange(newValue: string): void {
    this.value = newValue
    if (this._internals) this._internals.setFormValue(newValue)

    this.dispatchEvent(new Event('input', { bubbles: true, composed: true }))

    if (this._debounceTimer) clearTimeout(this._debounceTimer)
    this._debounceTimer = setTimeout(() => {
      this.dispatchEvent(new CustomEvent('ship-change', {
        detail: { value: newValue }, bubbles: true, composed: true,
      }))
    }, DEBOUNCE_MS)
  }

  focus(): void {
    this._view?.focus()
  }

  /**
   * Render a set of server-side diagnostics in the CodeMirror lint gutter.
   *
   * Pass an empty array to clear existing markers.
   *
   * Positions are 1-based. When `endLine`/`endColumn` are omitted, the
   * diagnostic spans from `(line, column)` to end-of-line so a marker
   * always has a visible range even when the server only gives us the
   * starting position of the offending value.
   */
  setDiagnostics(diagnostics: ShipDiagnostic[]): void {
    if (!this._view) {
      this._pendingDiagnostics = diagnostics
      return
    }
    this._applyDiagnostics(diagnostics)
  }

  private _applyDiagnostics(diagnostics: ShipDiagnostic[]): void {
    if (!this._view) return
    const state = this._view.state
    const doc = state.doc
    const cmDiagnostics: Diagnostic[] = diagnostics
      .map((d) => {
        const line = Math.max(1, Math.min(doc.lines, d.line | 0))
        const startLine = doc.line(line)
        const col = Math.max(1, (d.column ?? 1) | 0)
        const from = Math.min(
          startLine.to,
          startLine.from + col - 1,
        )
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
    this._view.dispatch(setDiagnostics(state, cmDiagnostics))
  }

  /** Replace the completion sources. Takes effect immediately — no
   *  re-render needed. Pass an empty array to disable. */
  setCompletions(sources: ShipCompletionSource[]): void {
    this._completions = sources ?? []
    if (this._view) {
      this._view.dispatch({
        effects: this._completionCompartment.reconfigure(
          this._buildCompletionExt(),
        ),
      })
    }
  }

  private _buildCompletionExt(): Extension {
    if (this._completions.length === 0) return []
    const sources = this._completions.map((s) =>
      makeCompletionSource(s),
    )
    return autocompletion({ override: sources, defaultKeymap: true })
  }

  render() {
    return html`<div class="editor-root" part="editor"></div>`
  }
}

/** Convert a declarative source into a CodeMirror CompletionSource.
 *  Matches the `trigger` text immediately before the cursor; when it's
 *  present, returns the items. CM handles prefix filtering against
 *  whatever the user has typed after the trigger. */
function makeCompletionSource(src: ShipCompletionSource) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const { pos, state } = ctx
    const trig = src.trigger
    const start = pos - trig.length
    if (start < 0) return null
    const preceding = state.sliceDoc(start, pos)
    if (preceding !== trig) {
      // Also allow a partial-word match after the trigger (e.g. `ship.ag`).
      const word = ctx.matchBefore(/[A-Za-z0-9_.]+/)
      if (!word) return null
      const before = state.sliceDoc(
        Math.max(0, word.from - trig.length),
        word.from,
      )
      if (before !== trig) return null
      return {
        from: word.from,
        options: src.items.map((it) => ({
          label: it.label,
          detail: it.detail,
          info: it.info,
          apply: it.insert ?? it.label,
        })),
      }
    }
    return {
      from: pos,
      options: src.items.map((it) => ({
        label: it.label,
        detail: it.detail,
        info: it.info,
        apply: it.insert ?? it.label,
      })),
    }
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-code-editor')) {
  customElements.define('ship-code-editor', ShipCodeEditor)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-code-editor': ShipCodeEditor
  }
}
