/**
 * <ship-markdown-editor> — Lit component wrapping TipTap v3. Edits markdown;
 * serializes via tiptap-markdown. Lazy-bundled (`bundle:editor`).
 *
 * Attrs: value, placeholder, size (sm|md|lg), tone, disabled, invalid, rows
 *   (default 12), full-height, name, label, show-selection-toolbar (true),
 *   allow-comment (true), allow-generate (false).
 * Events: input (bubbling, also fires once on mount for data-bind init),
 *   ship-change (debounced 150ms, detail { value }),
 *   ship-feedback (detail { action: 'comment', selection, comment } |
 *                        { action: 'generate', selection }).
 * Form-associated: setFormValue reports current markdown.
 */

import { LitElement, html, nothing } from 'lit'
import { shipMarkdownEditorStyles } from './ship-markdown-editor.styles'
import { shipMarkdownEditorBubbleStyles } from './ship-markdown-editor.bubble.styles'
import { createMarkdownEditor, readMarkdown, type MarkdownEditor } from './ship-markdown-editor.init'
import { renderBubble, renderComposer, type BubblePos, type PendingComment } from './ship-markdown-editor.bubble'
import { renderToolbar, type MarkMap } from './ship-markdown-editor.toolbar'
import type { Size, Tone } from './util'

export class ShipMarkdownEditor extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultValue = ''
  private editor: MarkdownEditor | null = null
  private editorHost: HTMLDivElement | null = null
  private shellEl: HTMLDivElement | null = null
  private changeTimer: ReturnType<typeof setTimeout> | null = null
  private syncTimer: ReturnType<typeof setTimeout> | null = null
  private suppressNextUpdate = false
  private marks: MarkMap = {
    bold: false, italic: false, strike: false, code: false, h1: false, h2: false,
    h3: false, bullet: false, ordered: false, task: false, link: false,
  }
  private bubblePos: BubblePos = null
  private pending: PendingComment = null

  constructor() { super(); this.internals = this.attachInternals() }

  static styles = [shipMarkdownEditorStyles, shipMarkdownEditorBubbleStyles]

  static properties = {
    value: { type: String },
    placeholder: { type: String },
    size: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    disabled: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
    rows: { type: Number },
    fullHeight: { type: Boolean, reflect: true, attribute: 'full-height' },
    name: { type: String },
    label: { type: String },
    showSelectionToolbar: { type: Boolean, attribute: 'show-selection-toolbar' },
    allowComment: { type: Boolean, attribute: 'allow-comment' },
    allowGenerate: { type: Boolean, attribute: 'allow-generate' },
  }

  value = ''
  placeholder = ''
  size: Size = 'md'
  tone?: Tone
  disabled = false
  invalid = false
  rows = 12
  fullHeight = false
  name = ''
  label = ''
  showSelectionToolbar = true
  allowComment = true
  allowGenerate = false

  connectedCallback(): void {
    super.connectedCallback()
    this.defaultValue = this.value
    this.applyMinHeight()
  }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this.changeTimer) clearTimeout(this.changeTimer)
    if (this.syncTimer) clearTimeout(this.syncTimer)
    this.changeTimer = this.syncTimer = null
    if (this.editor) { try { this.editor.destroy() } catch { /* teardown race */ } }
    this.editor = null
  }

  formResetCallback(): void {
    this.value = this.defaultValue
    this.editor?.commands.setContent(this.defaultValue)
    this.setFormValue(this.value)
  }

  formStateRestoreCallback(state: string | null): void {
    this.value = state ?? ''
    this.editor?.commands.setContent(this.value)
  }

  private applyMinHeight() {
    this.style.setProperty('--ed-min-h', `${Math.max(this.rows * 28, 160)}px`)
  }

  private setFormValue(v: string) {
    if (typeof this.internals.setFormValue === 'function') this.internals.setFormValue(v)
  }

  firstUpdated(): void {
    this.editorHost = this.renderRoot.querySelector('[data-editor-host]')
    this.shellEl = this.renderRoot.querySelector('[data-shell]')
    if (this.editorHost) void this.initEditor()
  }

  updated(changed: Map<string, unknown>): void {
    if (!this.editor) return
    if (changed.has('value')) {
      if (this.syncTimer) clearTimeout(this.syncTimer)
      this.syncTimer = setTimeout(() => this.syncExternalValue(), 50)
    }
    if (changed.has('disabled')) this.editor.setEditable(!this.disabled)
    if (changed.has('rows')) this.applyMinHeight()
    this.setFormValue(this.value)
  }

  private async initEditor(): Promise<void> {
    if (!this.editorHost) return
    this.editor = await createMarkdownEditor({
      element: this.editorHost, content: this.value, placeholder: this.placeholder,
      editable: !this.disabled,
      onUpdate: ed => this.onEditorUpdate(ed),
      onSelectionUpdate: () => { this.refreshMarks(); this.refreshBubble() },
      onTransaction: () => this.refreshMarks(),
      onBlur: () => this.hideBubble(),
    })
    this.refreshMarks()
    // Fire one input on init so data-bind consumers pick up the starting value.
    this.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    this.setFormValue(this.value)
  }

  private onEditorUpdate(ed: MarkdownEditor) {
    if (this.suppressNextUpdate) { this.suppressNextUpdate = false; return }
    const md = readMarkdown(ed)
    if (md === this.value) return
    this.value = md
    this.setFormValue(md)
    this.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    if (this.changeTimer) clearTimeout(this.changeTimer)
    this.changeTimer = setTimeout(() => {
      this.dispatchEvent(new CustomEvent('ship-change', {
        detail: { value: md }, bubbles: true, composed: true,
      }))
    }, 150)
  }

  private syncExternalValue() {
    if (!this.editor) return
    const current = readMarkdown(this.editor)
    if (current === this.value) return
    this.suppressNextUpdate = true
    this.editor.commands.setContent(this.value)
  }

  private refreshMarks() {
    const ed = this.editor; if (!ed) return
    this.marks = {
      bold: ed.isActive('bold'), italic: ed.isActive('italic'),
      strike: ed.isActive('strike'), code: ed.isActive('code'),
      h1: ed.isActive('heading', { level: 1 }), h2: ed.isActive('heading', { level: 2 }),
      h3: ed.isActive('heading', { level: 3 }), bullet: ed.isActive('bulletList'),
      ordered: ed.isActive('orderedList'), task: ed.isActive('taskList'),
      link: ed.isActive('link'),
    }
    this.requestUpdate()
  }

  private refreshBubble() {
    if (!this.showSelectionToolbar || this.disabled) { this.hideBubble(); return }
    const ed = this.editor
    const shell = this.shellEl
    if (!ed || !shell) { this.hideBubble(); return }
    const sel = ed.state.selection
    if (sel.empty || this.pending) { this.hideBubble(); return }
    try {
      const start = ed.view.coordsAtPos(sel.from)
      const end = ed.view.coordsAtPos(sel.to)
      const shellRect = shell.getBoundingClientRect()
      const top = Math.min(start.top, end.top) - shellRect.top
      const left = (start.left + end.right) / 2 - shellRect.left
      this.bubblePos = { top, left }
    } catch { this.bubblePos = null }
    this.requestUpdate()
  }

  private hideBubble() {
    if (this.bubblePos === null) return
    this.bubblePos = null
    this.requestUpdate()
  }

  private getSelectedText(): string {
    const ed = this.editor
    if (!ed) return ''
    const { from, to } = ed.state.selection
    return ed.state.doc.textBetween(from, to, ' ').trim()
  }

  private run = (fn: (ed: MarkdownEditor) => void) => (e: MouseEvent) => {
    e.preventDefault()
    if (this.editor && !this.disabled) { fn(this.editor); this.refreshBubble() }
  }

  private onLinkClick = (e: MouseEvent) => {
    e.preventDefault()
    if (!this.editor || this.disabled) return
    const prev = (this.editor.getAttributes('link').href as string | undefined) ?? ''
    const url = window.prompt('Link URL', prev); if (url === null) return
    const c = this.editor.chain().focus().extendMarkRange('link')
    url === '' ? c.unsetLink().run() : c.setLink({ href: url }).run()
  }

  private openComposer = () => {
    const text = this.getSelectedText()
    if (!text) return
    this.pending = { text, value: '' }
    this.hideBubble()
    this.requestUpdate()
    queueMicrotask(() => {
      const ta = this.renderRoot.querySelector<HTMLTextAreaElement>('[data-composer-input]')
      ta?.focus()
    })
  }

  private closeComposer = () => {
    this.pending = null
    this.requestUpdate()
    this.editor?.commands.focus()
  }

  private submitComposer = () => {
    const p = this.pending; if (!p) return
    const comment = p.value.trim()
    if (!comment) return
    this.dispatchEvent(new CustomEvent('ship-feedback', {
      detail: { action: 'comment', selection: p.text, comment },
      bubbles: true, composed: true,
    }))
    this.closeComposer()
  }

  private onGenerate = () => {
    const text = this.getSelectedText()
    if (!text) return
    this.dispatchEvent(new CustomEvent('ship-feedback', {
      detail: { action: 'generate', selection: text },
      bubbles: true, composed: true,
    }))
    this.hideBubble()
  }

  private onComposerInput = (e: Event) => {
    if (!this.pending) return
    this.pending = { ...this.pending, value: (e.target as HTMLTextAreaElement).value }
    this.requestUpdate()
  }

  private onComposerKey = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); this.submitComposer() }
    else if (e.key === 'Escape') { e.preventDefault(); this.closeComposer() }
  }

  render() {
    const tb = renderToolbar({ marks: this.marks, label: this.label, disabled: this.disabled,
      run: this.run, onLinkClick: this.onLinkClick })
    const bubble = renderBubble({ show: this.showSelectionToolbar, pos: this.bubblePos,
      pending: this.pending, marks: this.marks, allowComment: this.allowComment,
      allowGenerate: this.allowGenerate,
      handlers: { run: this.run, onComment: this.openComposer, onGenerate: this.onGenerate } })
    const composer = renderComposer({ pending: this.pending, pos: this.bubblePos,
      onInput: this.onComposerInput, onKeyDown: this.onComposerKey,
      onCancel: this.closeComposer, onSubmit: this.submitComposer })
    return html`<div class="shell" part="shell" data-shell
      aria-invalid=${this.invalid ? 'true' : nothing}
      aria-disabled=${this.disabled ? 'true' : nothing}>
      ${tb}
      <div class="surface" part="surface"><div data-editor-host part="editor"></div></div>
      ${bubble}${composer}
    </div>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-markdown-editor')) {
  customElements.define('ship-markdown-editor', ShipMarkdownEditor)
}

declare global {
  interface HTMLElementTagNameMap { 'ship-markdown-editor': ShipMarkdownEditor }
}
